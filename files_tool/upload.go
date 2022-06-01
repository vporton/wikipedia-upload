package files_tool

import (
	"log"
	"os"

	"files_tool/mydb"
)

func main() {
	db := mydb.openDB(os.Args[1])
	defer db.closeDB()

	file_number := db.getMinFileNumberToUpload()
	for {
		err := db.readFileData(file_number)
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
