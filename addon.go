package main

import (
	"archive/zip"
	"fmt"
	"io"
	"io/fs"
	"os"
	"path/filepath"
	"strings"
)

func addon() error {
	f, err := os.Create(fmt.Sprintf("%s/betterbedrocktranslate.mcpack", rootpath))
	if err != nil {
		return err
	}
	defer f.Close()
	z := zip.NewWriter(f)
	if err != nil {
		return err
	}
	defer z.Close()
	if err := filepath.Walk(fmt.Sprintf("%s/template/", rootpath), func(path string, info fs.FileInfo, err error) error {
		fmt.Println("pack up :", path)
		if err != nil {
			return err
		}
		if info.IsDir() {
			return nil
		}
		t, err := os.Open(path)
		if err != nil {
			return err
		}
		defer t.Close()
		zf, err := z.Create(strings.TrimPrefix(path, rootpath))
		if err != nil {
			return err
		}
		if _, err := io.Copy(zf, t); err != nil {
			return err
		}
		return nil
	}); err != nil {
		return err
	}
	return nil
}
