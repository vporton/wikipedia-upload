package files_tool

import (
	"log"
	"os"

	"files_tool/mydb"

	"golang.org/x/sync/semaphore"
)

func main() {
	db, err := mydb.OpenDB(os.Args[1])
	if err != nil {
		log.Println(err)
		os.Exit(1)
	}
	defer db.CloseDB()

	file_number, err := db.GetMinFileNumberToUpload()
	if err != nil {
		log.Println(err)
		os.Exit(1)
	}
	const maxWorkers = 5 // TODO: Make configurable
	sem := semaphore.NewWeighted(maxWorkers)
	for {
		fileData, err := db.ReadFileData(file_number)
		if err == mydb.ErrKeyNotFound {
			break
		}
		if err != nil {
			log.Println(err)
			os.Exit(1)
		}

		if err := sem.Acquire(ctx, 1); err != nil {
			log.Printf("Failed to acquire semaphore: %v", err)
			break
		}

		go func(i int) {
			defer sem.Release(1)
			hash, err = uploadFile(data)
			if err != nil {
				log.Println(err)
				os.Exit(1)
			}
			if hash != fileData.Hash {
				log.Println("File %s hash mismatch", fileData.Name)
				os.Exit(1)
			}
		}()

		// Acquire all of the tokens to wait for any remaining workers to finish.
		if err := sem.Acquire(ctx, int64(maxWorkers)); err != nil {
			log.Printf("Failed to acquire semaphore: %v", err)
			os.Exit(1)
		}
	}
}

// Returns the hash
func uploadFile(fileData *FileData) ([32]byte, error) {

}
