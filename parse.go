package main

import (
	"bufio"
	"errors"
	"os"
	"regexp"
	"strings"
)

var ann = regexp.MustCompile(`\s*#.*$`)

func parse(f *os.File) (map[string]string, error) {
	s, m := bufio.NewScanner(f), make(map[string]string)
	for s.Scan() {
		t := s.Text()
		if strings.HasPrefix(t, "#") || strings.HasPrefix(t, "\uFEFF#") || !strings.Contains(t, "=") {
			continue
		}
		kv := strings.SplitN(ann.ReplaceAllString(t, ""), "=", 2)
		if len(kv) != 2 {
			return nil, errors.New("illegal kv file")
		}
		m[kv[0]] = kv[1]
	}
	return m, nil
}
