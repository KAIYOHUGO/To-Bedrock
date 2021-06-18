package main

import (
	"fmt"
	"strings"
)

type counter struct {
	count uint64
	fn    func(uint64)
}

func printInPlace(s string) {
	fmt.Print(strings.Repeat(" ", 50))
	fmt.Printf("\r%s", s)
}

func newCounter(s func(uint64)) *counter {
	return &counter{
		count: 0,
		fn:    s,
	}
}

func (s *counter) Write(b []byte) (int, error) {
	n := len(b)
	s.count += uint64(n)
	s.fn(s.count)
	return n, nil
}
