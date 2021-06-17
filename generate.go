package main

import (
	"archive/zip"
	"encoding/json"
	"fmt"
	"io"
	"os"
)

func packAddon(langName string, engineVersion []int64, langKeyValue map[string]string) error {
	f, err := os.Create(fmt.Sprintf("./tobedrocktranslate_%s.mcpack", langName))
	if err != nil {
		return err
	}
	defer f.Close()
	z := zip.NewWriter(f)
	if err != nil {
		return err
	}
	defer z.Close()
	// create pack_icon
	{
		icon, err := z.Create("pack_icon.png")
		if err != nil {
			return err
		}
		fsIcon, err := fs.Open("assets/pack_icon.png")
		if err != nil {
			return err
		}
		if _, err := io.Copy(icon, fsIcon); err != nil {
			return err
		}
	}

	// create manifest file
	{
		man, err := z.Create("manifest.json")
		if err != nil {
			return err
		}
		if err := json.NewEncoder(man).Encode(Manifest{
			FormatVersion: 2,
			Header: Header{
				Description:      "translate tool made by kaiyo hugo",
				Name:             "better bedrock translate Resource Pack",
				UUID:             "66c6e9a8-3093-462a-9c36-dbb052165623",
				Version:          version,
				MinEngineVersion: engineVersion,
			},
			Modules: []Module{
				{
					Description: "translate pack",
					Type:        "resources",
					UUID:        "743f6949-53be-44b6-b326-398005028623",
					Version:     version,
				},
			},
		}); err != nil {
			return err
		}
	}

	// craete lang file
	{
		lang, err := z.Create(fmt.Sprintf("texts/%s.lang", langName))
		if err != nil {
			return err
		}
		for k, v := range langKeyValue {
			fmt.Fprintf(lang, "%s=%s\n", k, v)
		}
	}
	return nil
}
