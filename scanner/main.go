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

var rustFile = strings.Builder{}

//go:embed head.rs
var headFile string

func init() {
	rustFile.WriteString(headFile)
	rustFile.WriteString("\n")
}

func main() {
	workspace := filepath.Dir(getGoEnv("GOMOD"))
	outputFilename := filepath.Join(workspace, "..", "src", "structs.rs")

	consulPath := filepath.Join(workspace, "..", "consul")
	apiPath := filepath.Join(consulPath, "api")
	structsPath := filepath.Join(consulPath, "agent", "structs")

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
	}
	walkDir(apiPath, picks)

	picks = []string{
		// "HealthCheck",
		"HealthCheckDefinition",
		"CheckDefinition",
		"RaftIndex",
		"NodeService",
		"ServiceKind",
		"ServiceAddress",
		"Weights",
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
		"QueryOptions",
	}
	walkDir(structsPath, picks)

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
		structComment = strings.ReplaceAll(structComment, "\n", "\n/// ")

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

			// 检查是否为需要提取的结构体
			if !contains(picks, typeSpec.Name.Name) {
				continue
			}

			// if structSkip(typeSpec.Name.Name) {
			// 	continue
			// }

			if structComment != "" {
				rustFile.WriteString(fmt.Sprintf("/// %s\n", structComment))
			}

			// 输出结构体的名称和注释
			fmt.Printf("Struct Name: %s\n", typeSpec.Name.Name)
			rustFile.WriteString(structDerive(typeSpec.Name.Name))
			rustFile.WriteString(fmt.Sprintf("pub struct %s {\n", typeSpec.Name.Name))

			// 遍历结构体的字段
			for _, field := range structType.Fields.List {
				fieldName := ""
				fieldType := ""
				fieldRustType := ""
				fieldTag := ""
				fieldComment := ""
				fieldVec := 0
				fieldOptional := false

				// 提取字段注释
				if field.Doc != nil {
					fieldComment = strings.TrimSpace(field.Doc.Text())
					fieldComment = strings.ReplaceAll(fieldComment, "\n", "\n    /// ")
				}

				// 提取字段标签
				if field.Tag != nil {
					fieldTag = field.Tag.Value

					if strings.Contains(fieldTag, "omitempty") {
						fieldOptional = true
					}

					if strings.Contains(fieldTag, "json:\"-\"") {
						continue
					}
				}

				// 提取字段类型
				if field.Type != nil {
					se := unwrapSelectorExpr(field.Type)
					if se != nil {
						if isDuration(se) {
							fieldType = "time.Duration"
						} else if isTime(se) {
							fieldType = "time.Time"
						} else {
							fieldType = se.Sel.Name
							// if e, ok := field.Type.(*ast.Ident); ok {
							// 	// fmt.Printf("B %s\n", e.Name)
							// 	fieldType = e.Name
							// }
							// println("\tNot support this type: ", fieldType)
						}
					} else {
						vec, val := parseType(field.Type)
						if vec > 0 {
							fieldVec += vec
						}
						fieldType = val
						// fieldType = "*************************"

						// fmt.Printf("%v\n============================\n", fieldType)
						// fieldType = strings.Split(fieldType, " ")[0]
					}

					// for {
					// 	if l, found := strings.CutPrefix(fieldType, "[]"); found {
					// 		fieldType = l
					// 		fieldVec += 1
					// 		fieldType = strings.TrimPrefix(fieldType, "[]")
					// 		continue
					// 	}

					// 	break
					// }

					fieldRustType = toRustType(fieldType)
				}

				// 提取字段名称
				if field.Names != nil {
					fieldName = strings.TrimSpace(field.Names[0].Name)
				} else {
					fieldName = fieldType
				}

				if fieldSkip(typeSpec.Name.Name, fieldName) {
					continue
				}

				if fieldComment == "" {
					// rustFile.WriteString(fmt.Sprintf("  /// %s\n", fieldName))
				} else {
					rustFile.WriteString(fmt.Sprintf("    /// %s\n", fieldComment))
				}

				if fieldName == "Port" {
					fieldRustType = "u16"
				}

				if fieldRustType == "ServiceDefinition" {
					fieldRustType = "Box<ServiceDefinition>"
				}

				if fieldRustType == "ServiceConnect" {
					fieldRustType = "Box<ServiceConnect>"
				}

				if fieldRustType == "AgentServiceRegistration" {
					fieldRustType = "Box<AgentServiceRegistration>"
				}

				if fieldType == "QueryMeta" || fieldType == "QueryOptions" || fieldType == "WriteRequest" {
					fieldOptional = true
				}

				if fieldType == "EnterpriseMeta" {
					fieldOptional = true
					rustFile.WriteString("    #[cfg(feature = \"enterprise\")]\n")
				}

				rustFile.WriteString(fmt.Sprintf("    #[serde(rename = \"%s\")]\n", fieldName))

				typename := fieldRustType
				for i := 0; i < fieldVec; i++ {
					typename = fmt.Sprintf("Vec<%s>", typename)
				}
				if fieldOptional {
					rustFile.WriteString("    #[serde(skip_serializing_if = \"Option::is_none\")]\n")
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
				rustFile.WriteString(fmt.Sprintf("    pub %s: %s,\n\n", name, typename))
			}

			// end of struct
			rustFile.WriteString("}\n\n")

			// struct extra
			extraBody := structExtra(typeSpec.Name.Name)
			if extraBody != "" {
				rustFile.WriteString(extraBody)
			}
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
			toRustType(fmt.Sprintf("%s", e.Key)),
			toRustType(val),
		)
	}
	return 0, ""
}

func fieldSkip(structName, fieldName string) bool {
	if fieldName == "EnterpriseMeta" {
		return true
	}
	switch structName {
	default:
		return false
	case "ServiceDefinition":
		switch fieldName {
		default:
			return false
		case "Kind":
			return true
		}
	case "AgentServiceConnectProxyConfig":
		switch fieldName {
		default:
			return false
		case "Config":
			return true
		}
	case "ConnectProxyConfig":
		switch fieldName {
		default:
			return false
		case "Config":
			return true
		}
	case "Upstream":
		switch fieldName {
		default:
			return false
		case "Config":
			return true
		}
	case "EnvoyExtension":
		switch fieldName {
		default:
			return false
		case "Arguments":
			return true
		}
	}
}

// 指定 struct derive
func structDerive(name string) string {
	switch name {
	default:
		return "#[derive(Debug, Clone, Default, Serialize, Deserialize)]\n"
	case "Weights":
		return "#[derive(Debug, Clone, Serialize, Deserialize)]\n"
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
}` + "\n\n"
	case "HealthCheck":
		return "pub type HealthChecks = Vec<HealthCheck>;\n\n"
	case "AgentServiceCheck":
		return "pub type AgentServiceChecks = Vec<AgentServiceCheck>;\n\n"
	case "CheckType":
		return "pub type CheckTypes = Vec<CheckType>;\n\n"
	case "Upstream":
		return "pub type Upstreams = Vec<Upstream>;\n\n"
	case "ACLServiceIdentity":
		return "pub type ACLServiceIdentities = Vec<ACLServiceIdentity>;\n\n"
	case "ACLNodeIdentity":
		return "pub type ACLNodeIdentities = Vec<ACLNodeIdentity>;\n\n"
	case "ACLTemplatedPolicy":
		return "pub type ACLTemplatedPolicies = Vec<ACLTemplatedPolicy>;\n\n"
	case "ACLTokenListStub":
		return "pub type ACLTokenListStubs = Vec<ACLTokenListStub>;\n\n"
	case "ACLPolicyListStub":
		return "pub type ACLPolicyListStubs = Vec<ACLPolicyListStub>;\n\n"
	case "ACLAuthMethod":
		return "pub type ACLAuthMethods = Vec<ACLAuthMethod>;\n\n"
	case "ACLAuthMethodListStub":
		return "pub type ACLAuthMethodListStubs = Vec<ACLAuthMethodListStub>;\n\n"
	case "ACLBindingRule":
		return "pub type ACLBindingRules = Vec<ACLBindingRule>;\n\n"
	case "ACLRole":
		return "pub type ACLRoles = Vec<ACLRole>;\n\n"
	case "ACLToken":
		return "pub type ACLTokens = Vec<ACLToken>;\n\n"
	case "ACLPolicy":
		return "pub type ACLPolicies = Vec<ACLPolicy>;\n\n"
	}
}

func toRustType(name string) string {
	switch name {
	default:
		return name
	case "time.Duration":
		return "Option<String>"
	case "time.Time":
		return "Option<String>"
	case "string", "CheckID", "ServiceKind":
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

// func extractFromPath(path string) {
// buf, err := os.ReadFile(path)
// if err != nil {
// 	return
// }

// structs, err := extractStructs(buf)
// if err != nil {
// 	println(err)
// 	return
// }

// for _, item := range structs {
// rustFile += item.String()
// }
// }

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

// func extractStructs(buf []byte) ([]*parsedStruct, error) {
// 	fset := token.NewFileSet()
// 	file, err := parser.ParseFile(fset, "", buf, parser.ParseComments)
// 	if err != nil {
// 		return nil, err
// 	}
// 	extractor := &extractor{buffer: buf, structs: []*parsedStruct{}}
// 	ast.Walk(extractor, file)
// 	return extractor.structs, nil
// }

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

// here is the test
// func testToSnake(t *testing.T) {
// 	input := "MyLIFEIsAwesomE"
// 	want := "my_life_is_awesom_e"
// 	if got := toSnake(input); got != want {
// 		t.Errorf("ToSnake(%v) = %v, want %v", input, got, want)
// 	}
// }

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
