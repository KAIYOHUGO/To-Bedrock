package main

import (
	"embed"
	"encoding/json"
	"fmt"
	"os"
	"strconv"
	"strings"
)

//go:embed assets/*
var fs embed.FS

func main() {
	var (
		lang                        int
		version, j_e, j_c, b_e, b_c string
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
			Var:     &j_e,
		},
		{
			Message: "input bedrock version en_us lang file:",
			Omit:    false,
			Var:     &b_e,
		},
		{
			Message: "input java version lang file you want to translate to:",
			Omit:    false,
			Var:     &j_c,
		},
		{
			Message: "input bedrock version lang file to compose (can omit):",
			Omit:    true,
			Var:     &b_c,
		},
	}...)

	f_j_e, err := os.Open(j_e)
	if err != nil {
		panic(err)
	}
	m_j_e := make(map[string]string)
	json.NewDecoder(f_j_e).Decode(&m_j_e)
	f_j_e.Close()

	// swap k-v
	m_j_e = func() map[string]string {
		temp := make(map[string]string)
		for k, v := range m_j_e {
			temp[v] = k
		}
		return temp
	}()
	f_b_e, err := os.Open(b_e)
	if err != nil {
		panic(err)
	}
	f_j_c, err := os.Open(j_c)
	if err != nil {
		panic(err)
	}

	var f_b_c *os.File
	m_b_c := make(map[string]string)
	if b_c != "" {
		f_b_c, err = os.Open(b_c)
		if err != nil {
			panic(err)
		}
		if m_b_c, err = parse(f_b_c); err != nil {
			panic(err)
		}
	}

	// the lang to
	m_j_c := make(map[string]string)
	json.NewDecoder(f_j_c).Decode(&m_j_c)
	f_j_c.Close()
	m_b_e, err := parse(f_b_e)
	f_b_e.Close()
	if err != nil {
		panic(err)
	}
	var langKeyValue []string
	for k, v := range m_b_e {
		if el, ok := m_j_e[v]; ok {
			if m_b_c[k] != m_j_c[el] {
				langKeyValue = append(langKeyValue, fmt.Sprintf("%s=%s\n", k, m_j_c[el]))
			}
		}
	}
	var versionArray []int64
	for _, v := range strings.Split(version, ".") {
		number, err := strconv.Atoi(v)
		if err != nil {
			panic(err)
		}
		versionArray = append(versionArray, int64(number))
	}
	if err := addon(langlist[lang].(string), versionArray, langKeyValue); err != nil {
		panic(err)
	}
	println("done")
}
