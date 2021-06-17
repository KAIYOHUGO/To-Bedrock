package main

import (
	"bufio"
	"errors"
	"os"
	"regexp"
	"strings"
)

var ann = regexp.MustCompile(`\s*#.*$`)

func parseKeyValue(f *os.File) (map[string]string, error) {
	s, m := bufio.NewScanner(f), make(map[string]string)
	for s.Scan() {
		t := s.Text()
		// i hate UTF-8 BOM :(
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

// return bedrock-key:java-key key value
func bedrockJavaKeyValue(bedrock, java map[string]string) map[string]string {
	word, KeyValue := make(map[string]string), make(map[string]string)
	for k, v := range java {
		word[v] = k
	}
	for k, v := range bedrock {
		javaKey, ok := word[v]
		if ok {
			KeyValue[k] = javaKey
		}
	}
	return KeyValue
}

// remove the same value
func composeKeyValue(origin, compose map[string]string) map[string]string {
	for k, v := range origin {
		el, ok := compose[k]
		if ok && v == el {
			delete(origin, k)
		}
	}
	return origin
}
