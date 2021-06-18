package main

import (
	"archive/zip"
	"embed"
	"encoding/json"
	"fmt"
	"io"
	"io/ioutil"
	"net/http"
	"os"
	"strconv"
	"strings"
)

//go:embed assets/*
var fs embed.FS

func main() {
	var (
		lang, mode int
		// javaKey & translate is java json file,bedrockKey & compose is bedrock lang file
		version, javaKey, translate, bedrockKey, compose, javaPath string
		// zipTemp                                                    *zip.Reader
	)
	javaList := make(map[string]string)
	bedrockJavaMap := make(map[string]string)
	tempDir, err := ioutil.TempDir("", "")
	if err != nil {
		panic(err)
	}
	defer func() {
		os.RemoveAll(tempDir)
		os.Remove(tempDir)
	}()
	fmt.Println(tempDir)

	// ture==0
	InputSelect(Input{
		Message: "choice build mode (auto mode?):",
		Omit:    false,
		Var:     &mode,
		Select: []interface{}{
			"true",
			"false",
		},
	})

	InputText(Input{
		Message: "input version number (e.g. 1.17.0):",
		Omit:    false,
		Var:     &version,
	})
	var (
		versionArray []int64
		shortVersion string = strings.Join(strings.Split(version, ".")[:2], ".")
	)
	for _, v := range strings.Split(version, ".") {
		number, err := strconv.Atoi(v)
		if err != nil {
			panic(err)
		}
		versionArray = append(versionArray, int64(number))
	}

	if mode == 0 {
		InputText(Input{
			Message: "minecraft path https://minecraft.fandom.com/wiki/.minecraft/path:",
			Omit:    false,
			Var:     &javaPath,
		})
		printInPlace("parse java...")
		func() {
			file, err := os.Open(fmt.Sprintf("%s/assets/indexes/%s.json", javaPath, shortVersion))
			if err != nil {
				panic(err)
			}
			defer file.Close()
			var javaIndex JavaIndex
			json.NewDecoder(file).Decode(&javaIndex)
			for k, v := range javaIndex.Objects {
				if strings.HasPrefix(k, "minecraft/lang/") {
					javaList[strings.TrimSuffix(strings.TrimPrefix(k, "minecraft/lang/"), ".json")] = v.Hash
				}
			}
		}()
		func() {
			printInPlace("download bedrock lang file...")
			resp, err := http.Get(fmt.Sprintf("https://meedownloads.blob.core.windows.net/add-ons/Vanilla_Resource_Pack_%s.zip", version))
			if err != nil {
				panic(err)
			}
			defer resp.Body.Close()

			printInPlace("save bedrock lang file...")
			temp, err := os.CreateTemp(tempDir, "tb")
			if err != nil {
				panic(err)
			}
			defer temp.Close()
			io.CopyBuffer(temp, io.TeeReader(resp.Body, newCounter(func(u uint64) {
				printInPlace(fmt.Sprintf("downloading :%d", u))
			})), make([]byte, 1024*1024))
			zf, err := zip.OpenReader(temp.Name())
			if err != nil {
				panic(err)
			}
			defer zf.Close()

			// unzip lang file
			for _, v := range langlist {
				func() {
					printInPlace(fmt.Sprintf("parse %s...", v))
					zipLangFile, err := zf.Open(fmt.Sprintf("texts/%s.lang", v))
					if err != nil {
						panic(err)
					}
					defer zipLangFile.Close()
					tempLangFile, err := os.Create(fmt.Sprintf("%s/%s", tempDir, v))
					if err != nil {
						panic(err)
					}
					defer tempLangFile.Close()
					io.Copy(tempLangFile, zipLangFile)
				}()
			}
		}()

		func() {
			printInPlace("reading java lang file...")
			javaVersionFile, err := zip.OpenReader(fmt.Sprintf("%s/versions/%s/%s.jar", javaPath, shortVersion, shortVersion))
			if err != nil {
				panic(err)
			}
			defer javaVersionFile.Close()
			javaLangFile, err := javaVersionFile.Open("assets/minecraft/lang/en_us.json")
			if err != nil {
				panic(err)
			}
			defer javaLangFile.Close()
			javaTempFile, err := os.Create(fmt.Sprintf("%s/%s", tempDir, "java_en_us"))
			if err != nil {
				panic(err)
			}
			defer javaTempFile.Close()
			io.Copy(javaTempFile, javaLangFile)
		}()
		javaKey = fmt.Sprintf("%s/%s", tempDir, "java_en_us")
		bedrockKey = fmt.Sprintf("%s/%s", tempDir, "en_US")
		goto generate
	}

	InputSelect(Input{
		Message: "input lang type you want to translate to:",
		Omit:    false,
		Var:     &lang,
		Select:  langlist,
	})

	InputText([]Input{
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
			Message: fmt.Sprintf("input java version %s lang file:", langlist[lang]),
			Omit:    false,
			Var:     &translate,
		},
		{
			Message: fmt.Sprintf("input bedrock version %s lang file:", langlist[lang]),
			Omit:    true,
			Var:     &compose,
		},
	}...)

generate:
	// generate bedrock:java key-value
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

	packup := func() {
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
		if err := packAddon(langlist[lang].(string), versionArray, translated); err != nil {
			panic(err)
		}
		printInPlace(fmt.Sprintf("done %s", langlist[lang]))
	}
	if mode == 0 {
		for i, v := range langlist {
			printInPlace(fmt.Sprintf("running %s", langlist[lang]))
			if v == "en_US" {
				continue
			}
			hash, ok := javaList[strings.ToLower(v.(string))]
			if !ok {
				continue
			}
			lang = i
			translate = fmt.Sprintf("%s/assets/objects/%s/%s", javaPath, hash[:2], hash)
			compose = fmt.Sprintf("%s/%s", tempDir, v)
			packup()
		}
	} else {
		packup()
	}
	fmt.Println("\nall done, exit")
}
