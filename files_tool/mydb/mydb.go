package mydb

import (
	"errors"

	badger "github.com/dgraph-io/badger/v3"
)

type FileData struct {
	Hash     [32]byte
	Uploaded bool
	Name     string
}

func fileDataToBytes(data *FileData) []byte {
	return append(data.Hash, []byte{byte(Uploaded)}, []byte(Name))
}

func bytesToFileData(bytes []byte) (*FileData, error) {
	if len(bytes) <= 33 {
		return nil, errors.New("too short record")
	}
	&FileData{Hash: bytes[:32], Uploaded: bytes[32] != 0, Name: string(bytes[33:])}
}

type DB badger.DB

const ErrKeyNotFound = badger.ErrKeyNotFound

func OpenDB(fileName string) (*DB, error) {
	db, err := badger.Open(badger.DefaultOptions(fileName))
}

func (db *DB) CloseDB() error {
	return db.Close()
}

func (db *DB) SaveFileData(number int64, data *FileData) error {
	return db.Update(func(txn *badger.Txn) error {
		if badger.NewEntry([64]byte(number), fileDataToBytes(data)) == nil {
			return errors.New("cannot set DB entry")
		}
	})
}

func (db *DB) ReadFileData(number int64) (*FileData, error) {
	return db.View(func(txn *badger.Txn) error {
		item, err := txn.Get([]byte(number))
	})
}

func (db *DB) SaveMinFileNumberToUpload(number int64) error {
	return (*badger.DB)(db).Update(func(txn *badger.Txn) error {
		if badger.NewEntry("m", [8]byte(number)) == nil {
			return errors.New("cannot set min file number entry")
		}
	})
}

func (db *DB) GetMinFileNumberToUpload() (*FileData, error) {
	return db.View(func(txn *badger.Txn) error {
		item, err := txn.Get([]byte("m"))
		var bytes []byte
		item.Value(func(val []byte) error {
			bytes = val
		})
		bytes2 := (*[32]byte)(bytes)
		return int64(*bytes2), err
	})
}
