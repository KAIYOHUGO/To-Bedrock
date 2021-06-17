package main

import (
	"embed"
	"encoding/json"
	"os"
	"strconv"
	"strings"
)

//go:embed assets/*
var fs embed.FS

func main() {
	var (
		lang int
		// javaKey & translate is java json file,bedrockKey & compose is bedrock lang file
		version, javaKey, translate, bedrockKey, compose string
	)

	InputSelect(Input{
		Message: "input lang type you want to translate to:",
		Omit:    false,
		Var:     &lang,
		Select:  langlist,
	})

	InputText([]Input{
		{
			Message: "input version number (e.g. 1.17.0):",
			Omit:    false,
			Var:     &version,
		},
		{
			Message: "input java version en_us lang file:",
			Omit:    false,
			Var:     &javaKey,
		},
		{
			Message: "input bedrock version en_us lang file:",
			Omit:    false,
			Var:     &bedrockKey,
		},
		{
			Message: "input java version lang file you want to translate to:",
			Omit:    false,
			Var:     &translate,
		},
		{
			Message: "input bedrock version lang file to compose (can omit):",
			Omit:    true,
			Var:     &compose,
		},
	}...)

	// generate bedrock:java key-value
	bedrockJavaMap := make(map[string]string)
	func() {
		javaKeyFile, err := os.Open(javaKey)
		if err != nil {
			panic(err)
		}
		defer javaKeyFile.Close()
		bedrockKeyFile, err := os.Open(bedrockKey)
		if err != nil {
			panic(err)
		}
		defer bedrockKeyFile.Close()
		javaMap := make(map[string]string)
		if err := json.NewDecoder(javaKeyFile).Decode(&javaMap); err != nil {
			panic(err)
		}
		bedrockMap, err := parseKeyValue(bedrockKeyFile)
		if err != nil {
			panic(err)
		}
		bedrockJavaMap = bedrockJavaKeyValue(bedrockMap, javaMap)
	}()

	translated := make(map[string]string)
	func() {
		translateFile, err := os.Open(translate)
		if err != nil {
			panic(err)
		}
		defer translateFile.Close()
		translateMap := make(map[string]string)

		if err := json.NewDecoder(translateFile).Decode(&translateMap); err != nil {
			panic(err)
		}
		for k, v := range bedrockJavaMap {
			translated[k] = translateMap[v]
		}
	}()

	func() {
		if compose == "" {
			return
		}
		composeFile, err := os.Open(compose)
		if err != nil {
			panic(err)
		}
		defer composeFile.Close()
		composeMap, err := parseKeyValue(composeFile)
		if err != nil {
			panic(err)
		}
		translated = composeKeyValue(translated, composeMap)
	}()
	var versionArray []int64
	for _, v := range strings.Split(version, ".") {
		number, err := strconv.Atoi(v)
		if err != nil {
			panic(err)
		}
		versionArray = append(versionArray, int64(number))
	}
	if err := packAddon(langlist[lang].(string), versionArray, translated); err != nil {
		panic(err)
	}
	println("done")
}
