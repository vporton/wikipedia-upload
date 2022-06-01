package files_tool

import (
	"log"
	"os"

	"files_tool/mydb"
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
	for {
		data, err := db.ReadFileData(file_number)
		if err == mydb.ErrKeyNotFound {
			break
		}
		if err != nil {
			log.Println(err)
			os.Exit(1)
		}
		// TODO
	}
}
