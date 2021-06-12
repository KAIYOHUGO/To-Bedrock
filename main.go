package main

import (
	"bufio"
	"encoding/json"
	"fmt"
	"os"
	"path/filepath"
)

var rootpath string

func main() {
	var (
		lang, j_e, j_c, b_e, b_c string
		err                      error
	)

	rootpath, err = os.Executable()
	if err != nil {
		panic(err)
	}
	rootpath = filepath.Dir(rootpath)
lg:
	fmt.Print("input lang type you want to translate to:")
	fmt.Scanln(&lang)
	if lang == "" {
		goto lg
	}
je:
	fmt.Print("input java version en_us lang file:")
	fmt.Scanln(&j_e)
	if j_e == "" {
		goto je
	}
be:
	fmt.Print("input bedrock version en_us lang file:")
	fmt.Scanln(&b_e)
	if b_e == "" {
		goto be
	}
jc:
	fmt.Print("input java version lang file you want to translate to:")
	fmt.Scanln(&j_c)
	if j_c == "" {
		goto jc
	}
	fmt.Print("input bedrock version lang file to compose (can omit):")
	fmt.Scanln(&b_c)
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

	// output file
	o, err := os.Create(fmt.Sprintf("%s/template/texts/%s.lang", rootpath, lang))
	if err != nil {
		panic(err)
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
	w := bufio.NewWriter(o)
	for k, v := range m_b_e {
		if el, ok := m_j_e[v]; ok {
			if m_b_c[k] != m_j_c[el] {
				w.WriteString(fmt.Sprintf("%s=%s\n", k, m_j_c[el]))
			}
		}
	}
	w.Flush()
	o.Close()
	if err := addon(); err != nil {
		panic(err)
	}
	println("done")
}
