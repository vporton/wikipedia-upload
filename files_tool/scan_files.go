package scan_files

import (
	"fmt"
	"io"
	"io/fs"
	"log"
	"os"
	"path/filepath"

	"github.com/ethersphere/bee/pkg/swarm"

	"files_tool/mydb"
)

func main() {
	if len(os.Args) != 3 {
		log.Println("Usage: scan_files <DB-FILE> <DIR>")
		os.Exit(1)
	}

	db, err := mydb.OpenDB(os.Args[1])
	if err != nil {
		log.Println(err)
		os.Exit(1)
	}
	defer db.CloseDB()

	dir := os.Args[2]

	file_number := uint64(0)
	filepath.WalkDir(dir,
		func(path string, d fs.DirEntry, err error) error {
			if err != nil {
				log.Println(err)
				os.Exit(1)
			}
			if !d.IsDir() {
				relative_path, err := filepath.Rel(dir, path)
				if err != nil {
					log.Println(err)
					os.Exit(1)
				}
				hash, err := fileBZZHash(path)
				if err != nil {
					log.Println(err)
					os.Exit(1)
				}
				db.SaveFileData(file_number, &mydb.FileData{Hash: *(*[32]byte)(hash), Uploaded: false, Name: relative_path})
			}
			file_number += 1
			return nil
		})
	if err != nil {
		log.Println(err)
		os.Exit(1)
	}
	err = db.SaveMinFileNumberToUpload(0)
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
	stat, _ := os.Lstat(path)
	// chunker := storage.NewTreeChunker(storage.NewChunkerParams())
	// return chunker.Split(f, stat.Size(), nil, nil, nil)

	// fileStore := storage.NewFileStore(storage.NewMapChunkStore(), storage.NewFileStoreParams())
	// addr, _, err := fileStore.Store(f, stat.Size(), false)

	// FIXME: Is it correct code for Swarm hash?
	hasher := swarm.NewHasher()
	// _, err = hasher.Write([]byte("xxx")) // FIXME
	if stat.Mode()&os.ModeSymlink != 0 {
		link, err := os.Readlink(path)
		if err != nil {
			fmt.Println("Error opening file " + os.Args[1])
			os.Exit(1)
		}
		hasher.Write([]byte(link))
	} else {
		io.Copy(hasher, f)
	}
	addr := hasher.Sum(nil)

	return addr, err
}
