package main

import (
	_ "embed"
	"fmt"
	"go/ast"
	"go/parser"
	"go/token"
	"os"
	"os/exec"
	"path/filepath"
	"regexp"
	"strings"
)

//go:embed head.rs
var headFile string

var allStructs = make([]*rustStruct, 0)

type rustStructField struct {
	anonymous bool
	link      *rustStruct

	name     string
	typ      string
	cfg      []string
	comments []string
}

func (s *rustStructField) String() string {
	tmp := strings.Builder{}

	if s.anonymous {
		// 搬运匿名结构体内的所有字段
		for _, line := range s.link.fields {
			tmp.WriteString(line.String())
		}
	} else {
		for _, line := range s.comments {
			if line == "" {
				tmp.WriteString("    ///\n")
			} else {
				tmp.WriteString("    /// " + line + "\n")
			}
		}
		for _, line := range s.cfg {
			tmp.WriteString("    " + line + "\n")
		}

		tmp.WriteString("    pub " + s.name + ": " + s.typ + ",\n")
	}
	return tmp.String() + "\n"
}

type rustStruct struct {
	name     string
	derive   string
	comments []string
	fields   []*rustStructField
	extra    string
}

func (s *rustStruct) String() string {
	tmp := strings.Builder{}

	for i, line := range s.comments {
		if line == "" {
			if i > 0 {
				tmp.WriteString("///\n")
			}
		} else {
			tmp.WriteString("/// " + line + "\n")
		}
	}

	tmp.WriteString(s.derive + "\n")
	tmp.WriteString("pub struct " + s.name + " {\n")

	for _, line := range s.fields {
		tmp.WriteString(line.String())
	}

	tmp.WriteString("}\n")

	if s.extra != "" {
		tmp.WriteString("\n" + s.extra + "\n")
	}

	return tmp.String() + "\n"
}

func main() {
	workspace := filepath.Dir(getGoEnv("GOMOD"))

	consulPath := filepath.Join(workspace, "..", "consul-1.20.1")
	outputFilename := filepath.Join(workspace, "..", "src", "structs_1_20_x.rs")

	apiPath := filepath.Join(consulPath, "api")
	agentPath := filepath.Join(consulPath, "agent")
	structsPath := filepath.Join(agentPath, "structs")

	println("workspace: ", workspace)
	println("consulPath: ", consulPath)
	println("apiPath: ", apiPath)
	println("structsPath: ", structsPath)

	picks := []string{
		"AgentServiceChecksInfo",
		"AgentService",
		"HealthCheck",
		"AgentWeights",
		"AgentServiceConnectProxyConfig",
		"AgentServiceConnect",
		"AgentServiceRegistration",
		"AgentServiceCheck",
		"NamespaceACLConfig",
		"ACLLink",
		"QueryOptions",
	}
	walkDir(apiPath, picks)

	picks = []string{
		"UserEvent",
	}
	walkDir(agentPath, picks)

	picks = []string{
		"HealthCheckDefinition",
		"CheckDefinition",
		"NodeService",
		"RegisterRequest",
		"DeregisterRequest",
		"ServiceKind",
		"ServiceAddress",
		"ServiceNode",
		"ServiceName",
		"GatewayService",
		"Node",
		"NodeServices",
		"Weights",
		"CheckServiceNode",
		"Locality",
		"ConnectProxyConfig",
		"ServiceConnect",
		"ServiceDefinition",
		"PeeringServiceMeta",
		"ExposePath",
		"CheckType",
		"EnvoyExtension",
		"TransparentProxyConfig",
		"AccessLogsConfig",
		"ProxyMode",
		"Upstream",
		"MeshGatewayConfig",
		"ExposeConfig",
		"ConnectAuthorizeRequest",
		"WriteRequest",
	}
	walkDir(structsPath, picks)

	// 处理关联 anonymous 结构体
	for _, structItem := range allStructs {
		for _, fieldItem := range structItem.fields {
			if fieldItem.anonymous {
				found := false

			a:
				for _, lookup := range allStructs {
					if lookup.name == fieldItem.name {
						// println("关联结构体", lookup.name, fieldItem.name)
						// 关联
						fieldItem.link = lookup
						found = true
						break a
					}
				}

				if !found {
					println("关联结构体未找到,名称：" + structItem.name + ", 目标：" + fieldItem.name)
					return
				}
			}
		}
	}

	rustFile := strings.Builder{}
	rustFile.WriteString(headFile)
	rustFile.WriteString("\n")

	for _, structItem := range allStructs {
		rustFile.WriteString(structItem.String())
	}

	// 写入文件
	err := os.WriteFile(outputFilename, []byte(rustFile.String()), os.ModePerm)
	if err != nil {
		fmt.Println("Error writing file:", err)
		os.Exit(1)
	}
}

func parseGoFile(path string, picks []string) {
	fset := token.NewFileSet()
	node, err := parser.ParseFile(fset, path, nil, parser.ParseComments|parser.AllErrors)
	if err != nil {
		fmt.Println("Error parsing file:", err)
		os.Exit(1)
	}

	ast.Inspect(node, func(n ast.Node) bool {
		// 检查是否为结构体类型声明
		gd, ok := n.(*ast.GenDecl)
		if !ok || gd.Tok != token.TYPE {
			return true
		}

		structComment := strings.TrimSpace(gd.Doc.Text())
		structComments := strings.Split(structComment, "\n")

		for _, spec := range gd.Specs {
			typeSpec, ok := spec.(*ast.TypeSpec)
			if !ok {
				continue
			}

			// 检查是否为结构体
			structType, ok := typeSpec.Type.(*ast.StructType)
			if !ok {
				continue
			}

			structName := typeSpec.Name.Name

			// 检查是否为需要提取的结构体
			if !contains(picks, structName) {
				continue
			}

			structTemp := rustStruct{
				name:     structName,
				derive:   structDerive(structName),
				comments: structComments,
				fields:   make([]*rustStructField, 0),
				extra:    structExtra(structName),
			}

		a:
			// 遍历结构体的字段
			for _, field := range structType.Fields.List {
				fieldName := ""
				fieldType := ""
				fieldRustType := ""
				fieldTag := ""
				fieldComment := ""
				fieldComments := []string{}
				fieldVec := 0
				fieldOptional := false

				// 提取字段标签
				if field.Tag != nil {
					fieldTag = field.Tag.Value

					if strings.Contains(fieldTag, "json:\"-\"") {
						continue a
					}

					if strings.Contains(fieldTag, "omitempty") {
						fieldOptional = true
					}
				}

				// 提取字段注释
				if field.Doc != nil {
					fieldComment = strings.TrimSpace(field.Doc.Text())
					fieldComments = strings.Split(fieldComment, "\n")
				}

				// 提取字段类型
				if field.Type != nil {
					se := unwrapSelectorExpr(field.Type)
					if se != nil {
						if isDuration(se) {
							fieldOptional = true
							fieldType = "time.Duration"
						} else if isTime(se) {
							fieldOptional = true
							fieldType = "time.Time"
						} else {
							fieldType = se.Sel.Name
						}
					} else {
						vec, val := parseType(field.Type)
						if vec > 0 {
							fieldVec += vec
						}
						fieldType = val
					}

					fieldRustType = toRustType(fieldType)
				}

				// 提取字段名称
				if field.Names != nil {
					fieldName = strings.TrimSpace(field.Names[0].Name)
					if fieldSkip(structName, fieldName) {
						continue a
					}
				} else {
					if fieldSkip(structName, fieldType) {
						continue a
					}
					// 匿名字段
					structTemp.fields = append(structTemp.fields, &rustStructField{
						anonymous: true,
						name:      fieldType,
					})
					continue a
				}

				fieldTmp := rustStructField{}

				if fieldName == "AggregatedStatus" {
					fieldRustType = "Health"
				}

				if strings.Contains(fieldName, "Port") {
					fieldRustType = "u16"
				} else if fieldRustType == "ServiceDefinition" {
					fieldRustType = "Box<ServiceDefinition>"
				} else if fieldRustType == "ServiceConnect" {
					fieldRustType = "Box<ServiceConnect>"
				} else if fieldRustType == "AgentServiceRegistration" {
					fieldRustType = "Box<AgentServiceRegistration>"
				} else if fieldRustType == "HealthCheckDefinition" {
					fieldOptional = true
				}

				if structName == "WriteRequest" && fieldName == "Token" {
					fieldOptional = true
				}

				// UserEvent.Payload 虽然是 []byte，但会进行 base64 编码为 string
				if structName == "UserEvent" && fieldName == "Payload" {
					fieldOptional = true
					fieldVec = 0
					fieldRustType = "Base64Payload"
				}

				if structName == "RegisterRequest" {
					switch fieldName {
					case "TaggedAddresses", "NodeMeta", "Service", "Check", "Checks":
						fieldOptional = true
					}
				}

				if structName == "NodeService" {
					switch fieldName {
					case "Proxy", "Connect":
						fieldOptional = true
					}
				}

				if fieldType == "QueryMeta" || fieldType == "QueryOptions" || fieldType == "WriteRequest" {
					fieldOptional = true
				}

				if fieldType == "EnterpriseMeta" || fieldType == "Locality" {
					fieldOptional = true
					fieldTmp.cfg = append(fieldTmp.cfg, "#[cfg(feature = \"enterprise\")]")
				}

				fieldTmp.cfg = append(fieldTmp.cfg, fmt.Sprintf("#[serde(rename = \"%s\")]", fieldName))

				typename := fieldRustType
				for i := 0; i < fieldVec; i++ {
					typename = fmt.Sprintf("Vec<%s>", typename)
				}
				if fieldOptional {
					fieldTmp.cfg = append(fieldTmp.cfg, "#[serde(skip_serializing_if = \"Option::is_none\")]")
					typename = fmt.Sprintf("Option<%s>", typename)
				}
				name := toSnake(fieldName)
				if name == "type" {
					name = "r#type"
				} else if name == "match" {
					name = "r#match"
				} else if name == "override" {
					name = "r#override"
				}

				fieldTmp.name = name
				fieldTmp.comments = fieldComments
				fieldTmp.typ = typename

				structTemp.fields = append(structTemp.fields, &fieldTmp)
			}

			allStructs = append(allStructs, &structTemp)
		}
		return true
	})
}

func parseType(t ast.Expr) (int, string) {
	if e, ok := t.(*ast.StarExpr); ok {
		return 0, e.X.(*ast.Ident).Name
	}

	if e, ok := t.(*ast.Ident); ok {
		return 0, e.Name
	}

	if e, ok := t.(*ast.ArrayType); ok {
		vec, val := parseType(e.Elt)
		return 1 + vec, val
	}
	if e, ok := t.(*ast.MapType); ok {
		vec, val := parseType(e.Value)
		if vec > 0 {
			val := toRustType(val)
			for i := 0; i < vec; i++ {
				val = fmt.Sprintf("Vec<%s>", val)
			}
			return 0, fmt.Sprintf(
				"::std::collections::HashMap<%s, %s>",
				toRustType(fmt.Sprintf("%s", e.Key)),
				val,
			)
		}
		return vec, fmt.Sprintf(
			"::std::collections::HashMap<%s, %s>",
			toRustType(fmt.Sprintf("%s", e.Key)), toRustType(val),
		)
	}
	return 0, ""
}

func fieldSkip(structName, fieldName string) bool {
	// 小写开头则跳过
	if fieldName[0] >= 'a' && fieldName[0] <= 'z' {
		return true
	}
	if fieldName == "EnterpriseMeta" {
		return true
	}
	if fieldName == "RaftIndex" {
		return true
	}
	if fieldName == "PeerName" {
		return true
	}
	switch structName {
	case "ServiceDefinition", "AgentService":
		switch fieldName {
		case "Kind":
			return true
		}
	case "NodeService":
		switch fieldName {
		case "Kind", "PeerName":
			return true
		}
	case "AgentServiceConnectProxyConfig":
		switch fieldName {
		case "Config":
			return true
		}
	case "RegisterRequest":
		switch fieldName {
		case "PeerName":
			return true
		}
	case "ConnectProxyConfig":
		switch fieldName {
		case "Config":
			return true
		}
	case "Upstream":
		switch fieldName {
		case "Config":
			return true
		}
	case "EnvoyExtension":
		switch fieldName {
		case "Arguments":
			return true
		}
	}
	return false
}

// 指定 struct derive
func structDerive(name string) string {
	switch name {
	default:
		return "#[derive(Debug, Clone, Default, Serialize, Deserialize)]"
	case "Weights":
		return "#[derive(Debug, Clone, Serialize, Deserialize)]"
	}
}

// struct 的自定义扩展内容
func structExtra(name string) string {
	switch name {
	default:
		return ""
	case "Weights":
		return `/// Specifies weights for the service.
/// Default is {"Passing": 1, "Warning": 1}.
/// Learn more: https://developer.hashicorp.com/consul/api-docs/agent/service#weights
impl Default for Weights {
    fn default() -> Self {
        Self {
            passing: 1,
            warning: 1,
        }
    }
}` + "\n"
	case "HealthCheck":
		return "pub type HealthChecks = Vec<HealthCheck>;\n"
	case "AgentServiceCheck":
		return "pub type AgentServiceChecks = Vec<AgentServiceCheck>;\n"
	case "CheckType":
		return "pub type CheckTypes = Vec<CheckType>;\n"
	case "Upstream":
		return "pub type Upstreams = Vec<Upstream>;\n"
	case "ACLServiceIdentity":
		return "pub type ACLServiceIdentities = Vec<ACLServiceIdentity>;\n"
	case "ACLNodeIdentity":
		return "pub type ACLNodeIdentities = Vec<ACLNodeIdentity>;\n"
	case "ACLTemplatedPolicy":
		return "pub type ACLTemplatedPolicies = Vec<ACLTemplatedPolicy>;\n"
	case "ACLTokenListStub":
		return "pub type ACLTokenListStubs = Vec<ACLTokenListStub>;\n"
	case "ACLPolicyListStub":
		return "pub type ACLPolicyListStubs = Vec<ACLPolicyListStub>;\n"
	case "ACLAuthMethod":
		return "pub type ACLAuthMethods = Vec<ACLAuthMethod>;\n"
	case "ACLAuthMethodListStub":
		return "pub type ACLAuthMethodListStubs = Vec<ACLAuthMethodListStub>;\n"
	case "ACLBindingRule":
		return "pub type ACLBindingRules = Vec<ACLBindingRule>;\n"
	case "ACLRole":
		return "pub type ACLRoles = Vec<ACLRole>;\n"
	case "ACLToken":
		return "pub type ACLTokens = Vec<ACLToken>;\n"
	case "ACLPolicy":
		return "pub type ACLPolicies = Vec<ACLPolicy>;\n"
	}
}

func toRustType(name string) string {
	switch name {
	default:
		return name
	case "time.Duration":
		return "String"
	case "time.Time":
		return "String"
	case "string", "CheckID", "NodeID", "ServiceKind", "ProxyMode":
		return "String"
	case "int":
		return "isize"
	case "int8":
		return "i8"
	case "int16":
		return "i16"
	case "int32":
		return "i32"
	case "int64":
		return "i64"
	case "uint":
		return "usize"
	case "uint8", "byte":
		return "u8"
	case "uint16":
		return "u16"
	case "uint32":
		return "u32"
	case "uint64":
		return "u64"
	case "bool":
		return "bool"
	}
}

func walkDir(path string, picks []string) {
	if err := filepath.Walk(path, func(path string, info os.FileInfo, err error) error {
		if err != nil {
			return err
		}
		if info.IsDir() {
			return nil
		}
		if filepath.Ext(path) != ".go" {
			return nil
		}
		// Don't extract from test files.
		if strings.HasSuffix(path, "_test.go") {
			return nil
		}

		parseGoFile(path, picks)

		return nil
	}); err != nil {
		return
	}
}

var goEnvCache = ""

func getGoEnv(key string) string {
	if goEnvCache != "" {
		return goEnvCache
	}
	out, err := exec.Command("go", "env", key).Output()
	if err != nil {
		panic(err.Error())
	}
	goEnvCache = string(out)
	return string(out)
}

var matchNonAlphaNumeric = regexp.MustCompile(`[^a-zA-Z0-9]+`)
var matchFirstCap = regexp.MustCompile("(.)([A-Z][a-z]+)")
var matchAllCap = regexp.MustCompile("([a-z0-9])([A-Z])")

func toSnake(camel string) string {
	camel = matchNonAlphaNumeric.ReplaceAllString(camel, "_")   //非常规字符转化为 _
	snake := matchFirstCap.ReplaceAllString(camel, "${1}_${2}") //拆分出连续大写
	snake = matchAllCap.ReplaceAllString(snake, "${1}_${2}")    //拆分单词
	return strings.ToLower(snake)                               //全部转小写
}

func isTime(se *ast.SelectorExpr) bool {
	if se.Sel.Name != "Time" {
		return false
	}
	x, ok := se.X.(*ast.Ident)
	if !ok {
		return false
	}
	return x.Name == "time"
}

func isDuration(se *ast.SelectorExpr) bool {
	if se.Sel.Name != "Duration" {
		return false
	}
	x, ok := se.X.(*ast.Ident)
	if !ok {
		return false
	}
	return x.Name == "time"
}

func unwrapSelectorExpr(e ast.Expr) *ast.SelectorExpr {
	switch et := e.(type) {
	case *ast.SelectorExpr:
		return et
	case *ast.StarExpr:
		se, _ := et.X.(*ast.SelectorExpr)
		return se
	default:
		return nil
	}
}

func contains(s []string, e string) bool {
	for _, a := range s {
		if a == e {
			return true
		}
	}
	return false
}
