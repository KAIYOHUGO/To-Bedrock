package main

import "fmt"

type Input struct {
	Message interface{}
	Omit    bool
	// need be a string pointer in text,
	// need be a int pointer in select
	Var    interface{}
	Select []interface{}
}

// just input text
func InputText(input ...Input) {
	for _, v := range input {
	retry:
		fmt.Print(v.Message)
		fmt.Scanln(v.Var)
		if *v.Var.(*string) == "" && !v.Omit {
			goto retry
		}
	}
}

// input number
func InputSelect(input ...Input) {
	for _, v := range input {
		for i, l := range v.Select {
			fmt.Println("(", i+1, "):", l)
		}
	retry:
		fmt.Print(v.Message)
		fmt.Scanln(v.Var)
		if (v.Var == nil || *v.Var.(*int) <= 0 || *v.Var.(*int) >= len(v.Select)) && !v.Omit {
			goto retry
		}
		*v.Var.(*int)--
	}
}
