package main

import (
	"flag"
	"fmt"
	"io"
	"os"

	"github.com/dethrandir/markstone/bindings/go"
)

func main() {
	modeFlag := flag.String("mode", "", "Conversion mode: generic-html, generic-ast, actos-html, actos-ast")
	flag.Parse()

	if *modeFlag == "" {
		fmt.Fprintln(os.Stderr, "Error: --mode flag is required")
		os.Exit(1)
	}

	var input []byte
	var err error

	args := flag.Args()
	if len(args) > 0 && args[0] != "" && args[0] != "-" {
		input, err = os.ReadFile(args[0])
		if err != nil {
			fmt.Fprintf(os.Stderr, "Error reading input file %s: %v\n", args[0], err)
			os.Exit(1)
		}
	} else {
		input, err = io.ReadAll(os.Stdin)
		if err != nil {
			fmt.Fprintf(os.Stderr, "Error reading stdin: %v\n", err)
			os.Exit(1)
		}
	}

	var output string
	switch *modeFlag {
	case "generic-html":
		output, err = markstone.ToHTMLBytes(input)
	case "generic-ast":
		output, err = markstone.ToASTBytes(input)
	case "actos-html":
		output, err = markstone.ActosToHTMLBytes(input)
	case "actos-ast":
		output, err = markstone.ActosToASTBytes(input)
	default:
		fmt.Fprintf(os.Stderr, "Unknown mode: %s\n", *modeFlag)
		os.Exit(1)
	}

	if err != nil {
		fmt.Fprintf(os.Stderr, "Conversion error: %v\n", err)
		os.Exit(1)
	}

	if _, err := os.Stdout.WriteString(output); err != nil {
		fmt.Fprintf(os.Stderr, "Write error: %v\n", err)
		os.Exit(1)
	}
}
