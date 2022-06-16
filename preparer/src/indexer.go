package main

import (
	"io/fs"
	"log"
	"os"
	"path/filepath"
)

func main() {
	logger := log.Default()
	if err := almost_main(logger); err != nil {
		logger.Fatal(err)
	}
}

func almost_main(logger *log.Logger) error {
	// FIXME: Parse command line
	os.Mkdir(args.output_dir.clone())
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
			if err := index_file(path, &args)?;; err != nil {
				return err
			}
		}
		return nil
	})
	return nil
}
