package main

import (
	"io"
	"io/fs"
	"io/ioutil"
	"log"
	"os"
	"path/filepath"

	"github.com/andybalholm/brotli"
)

func main() {
	logger := log.Default()
	if err := almost_main(logger); err != nil {
		logger.Fatal(err)
	}
}

func almost_main(logger *log.Logger) error {
	if len(os.Args) != 2 {
		logger.Fatal("Usage: brotler <DIR>")
	}
	filepath.WalkDir(os.Args[1], func(path string, d fs.DirEntry, err error) error {
		if err != nil {
			return err
		}
		info, err2 := d.Info()
		if err2 != nil {
			return err2
		}
		if info.Mode()&os.ModeSymlink == 0 && !info.IsDir() {
			logger.Print("Compressing file ", path)
			if err := compressFile(path); err != nil {
				return err
			}
		}
		return nil
	})
	return nil
}

func compressFile(path string) error {
	input, err0 := os.Open(path)
	if err0 != nil {
		return err0
	}
	output, err := ioutil.TempFile("/tmp", "brotler")
	if err != nil {
		return err
	}

	encoder := brotli.NewWriterLevel(output, brotli.BestCompression)
	io.Copy(encoder, input)
	encoder.Close()
	output_rev, err2 := os.Open(output.Name())
	if err2 != nil {
		return err2
	}
	input_rev, err3 := os.Create(path)
	if err3 != nil {
		return err3
	}
	io.Copy(input_rev, output_rev)
	return nil
}
