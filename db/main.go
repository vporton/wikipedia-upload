package main

import (
	"fmt"
	"os"
	"io"
	"github.com/ethersphere/bee/pkg/swarm"
)

func main() {
}

func fileBZZHash(path string) ([]byte, error) {
	f, err := os.Open(path)
	if err != nil {
		fmt.Println("Error opening file " + os.Args[1])
		os.Exit(1)
	}
	// stat, _ := f.Stat()
	// chunker := storage.NewTreeChunker(storage.NewChunkerParams())
	// return chunker.Split(f, stat.Size(), nil, nil, nil)

	// fileStore := storage.NewFileStore(storage.NewMapChunkStore(), storage.NewFileStoreParams())
	// addr, _, err := fileStore.Store(f, stat.Size(), false)

	// FIXME: Is it correct code for Swarm hash?
	hasher := swarm.NewHasher()
	// _, err = hasher.Write([]byte("xxx")) // FIXME
	io.Copy(hasher, f)
	addr := hasher.Sum(nil)

	return addr, err
}
