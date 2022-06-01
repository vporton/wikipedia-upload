package main

import (
	"fmt"
	"os"
	"io"
	"path/filepath"
	"github.com/ethersphere/bee/pkg/swarm"

	"wikipedia_upload/mydb"
)

func main() {
	db := mydb.openDB(os.Args[1])
	defer db.closeDB()

	const dir = os.Args[2];

	file_number := 0
	filepath.WalkDir(dir,
		func(path string, info os.FileInfo, err error) error {
			if err != nil {
				log.Println(err)
				os.Exit(1)
			}
			if !info.IsDir() {
				relative_path := Rel(dir, path)
				hash, err := fileBZZHash(path)
				if err != nil {
					log.Println(err)
					os.Exit(1)
				}
				db.saveFileData(FileData{hash: hash, uploaded: false, name: relative_path})
			}
			file_number += 1
			return nil
		})
	if err != nil {
		log.Println(err)
		os.Exit(1)
	}
	err = db.saveMinFileNumberToUpload(0)
	if err != nil {
		log.Println(err)
		os.Exit(1)
	}
}

func fileBZZHash(path string) ([]byte, error) {
	f, err := os.Open(path)
	if err != nil {
		fmt.Println("Error opening file " + os.Args[1])
		os.Exit(1)
	}
	stat, _ := f.LStat()
	// chunker := storage.NewTreeChunker(storage.NewChunkerParams())
	// return chunker.Split(f, stat.Size(), nil, nil, nil)

	// fileStore := storage.NewFileStore(storage.NewMapChunkStore(), storage.NewFileStoreParams())
	// addr, _, err := fileStore.Store(f, stat.Size(), false)

	// FIXME: Is it correct code for Swarm hash?
	hasher := swarm.NewHasher()
	// _, err = hasher.Write([]byte("xxx")) // FIXME
	if stat.IsLink() {
		hasher.Write(os.Readlink(path))
	} else {
		io.Copy(hasher, f)
	}
	addr := hasher.Sum(nil)

	return addr, err
}
