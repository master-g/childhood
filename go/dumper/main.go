package main

import (
	"fmt"
	"os"
)

func checkErr(err error) {
	if err != nil {
		fmt.Println(err)
		os.Exit(-1)
	}
}

func main() {
	if len(os.Args) < 2 {
		fmt.Println("usage: dumper rom")
		os.Exit(0)
	}

	f, err := os.Open(os.Args[1])
	defer f.Close()
	checkErr(err)

	if err = ExtractROM(os.Args[1]); err != nil {
		fmt.Println(err)
		os.Exit(1)
	}
}
