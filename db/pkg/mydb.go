import (
	badger "github.com/dgraph-io/badger/v3"
)

struct FileData {
	hash [32]byte
	uploaded bool
	name string
}

func fileDataToBytes(data *FileData) []byte {
	return append(data.hash, { byte(uploaded) }, []byte(name))
}

func bytesToFileData(bytes []byte) *FileData, error {
	if len(bytes) <= 33 {
		return nil, errors.New("too short record")
	}
	&FileData{hash: bytes[:32], uploaded: bytes[32] != 0, name = string(bytes[33:])}
}

type DB badger.DB

func openDB(fileName string) DB, error {
	db, err := badger.Open(badger.DefaultOptions(fileName))
}

func closeDB(db *DB) error {
	return db.Close()
}

func saveFileData(db DB, number int64, data *FileData) error {
	return db.Update(func(txn *badger.Txn) error {
		if badger.NewEntry([64]byte(number), fileDataToBytes(data)) = nil {
			return errors.New("cannot set DB entry")
		}
	)
}

func readFileData(db DB, number int64) *FileData, error {
	return db.View(func(txn *badger.Txn) error {
		item, err := txn.Get([]byte(number))
	})
}
