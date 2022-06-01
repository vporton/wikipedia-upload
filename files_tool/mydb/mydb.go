package mydb

import (
	badger "github.com/dgraph-io/badger/v3"
)

type FileData struct {
	hash [32]byte
	uploaded bool
	name string
}

func fileDataToBytes(data *FileData) []byte {
	return append(data.hash, []byte{ byte(uploaded) }, []byte(name))
}

func bytesToFileData(bytes []byte) *FileData, error {
	if len(bytes) <= 33 {
		return nil, errors.New("too short record")
	}
	&FileData{hash: bytes[:32], uploaded: bytes[32] != 0, name = string(bytes[33:])}
}

type DB badger.DB

const ErrKeyNotFound = badger.ErrKeyNotFound

func OpenDB(fileName string) *DB, error {
	db, err := badger.Open(badger.DefaultOptions(fileName))
}

func(db *DB) CloseDB() error {
	return db.Close()
}

func(db *DB) SaveFileData(number int64, data *FileData) error {
	return db.Update(func(txn *badger.Txn) error {
		if badger.NewEntry([64]byte(number), fileDataToBytes(data)) = nil {
			return errors.New("cannot set DB entry")
		}
	)
}

func(db *DB) ReadFileData(number int64) *FileData, error {
	return db.View(func(txn *badger.Txn) error {
		item, err := txn.Get([]byte(number))
	})
}

func(db *DB) SaveMinFileNumberToUpload(number int64) error {
	return db.Update(func(txn *badger.Txn) error {
		if badger.NewEntry("m", [8]byte(number)) = nil {
			return errors.New("cannot set min file number entry")
		}
	)
}

func(db *DB) getMinFileNumberToUpload(number int64) *FileData, error {
	return db.View(func(txn *badger.Txn) error {
		item, err := txn.Get([]byte("m"))
	})
}
