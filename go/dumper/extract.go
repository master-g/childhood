package main

import (
	"io"
	"os"
	"path"
	"strings"

	"fmt"

	"github.com/btcsuite/goleveldb/leveldb/errors"
)

var (
	ErrorInvalidHeader = errors.New("invalid NES header")
	ErrorInvalidROM    = errors.New("invalid NES rom")
)

func makeOutputDir(f string) (string, error) {
	_, fname := path.Split(f)
	pos := strings.Index(strings.ToLower(fname), ".nes")
	if pos != -1 {
		fname = fname[:pos]
	}
	if _, err := os.Stat(fname); os.IsNotExist(err) {
		return fname, os.Mkdir(fname, 0700)
	} else {
		return fname, nil
	}
}

func makeFile(p string) (*os.File, error) {
	w, err := os.Create(p)
	if err != nil {
		return nil, err
	}
	return w, nil
}

func extractSection(r io.Reader, outputPath string, size int) error {
	w, err := makeFile(outputPath)
	defer w.Close()
	if err != nil {
		return err
	}

	buf := make([]byte, size)
	n, err := r.Read(buf)
	if err != nil {
		return err
	}
	if n != size {
		return ErrorInvalidROM
	}
	n, err = w.Write(buf)
	if err != nil {
		return err
	}
	if n != size {
		return ErrorInvalidROM
	}
	w.Sync()

	return nil
}

func ExtractROM(romFile string) error {
	// open NES Rom
	r, err := os.Open(romFile)
	defer r.Close()
	if err != nil {
		return err
	}
	// create output dir
	outputDir, err := makeOutputDir(romFile)
	if err != nil {
		return err
	}
	// header
	header := NewHeader(r)
	if header == nil {
		return ErrorInvalidHeader
	}
	fmt.Println(header)

	// trainer
	if header.Trainer() {
		err := extractSection(r, path.Join(outputDir, "TRAINER.bin"), 512)
		if err != nil {
			return err
		}
	}
	if header.PRGROMSize() != 0 {
		err := extractSection(r, path.Join(outputDir, "PRGROM.bin"), header.PRGROMSize())
		if err != nil {
			return err
		}
	}
	if header.CHRROMSize() != 0 {
		err := extractSection(r, path.Join(outputDir, "CHRROM.bin"), header.CHRROMSize())
		if err != nil {
			return err
		}
	}

	return nil
}
