package mydb

import (
	"encoding/binary"
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

var ErrKeyNotFound = badger.ErrKeyNotFound

func OpenDB(fileName string) (*DB, error) {
	return badger.Open(badger.DefaultOptions(fileName))
}

func (db *DB) CloseDB() error {
	return (*badger.DB)(db).Close()
}

func (db *DB) SaveFileData(number uint64, data *FileData) error {
	return (*badger.DB)(db).Update(func(txn *badger.Txn) error {
		if badger.NewEntry([64]byte(number), fileDataToBytes(data)) == nil {
			return errors.New("cannot set DB entry")
		}
	})
}

func (db *DB) ReadFileData(number uint64) (*FileData, error) {
	var item *badger.Item
	err := (*badger.DB)(db).View(func(txn *badger.Txn) error {
		b := make([]byte, 8)
		binary.LittleEndian.PutUint64(b, number)

		item, err = txn.Get(b)
	})
	if err != nil {
		return nil, err
	}
	var bytes []byte
	item.Value(func(val []byte) error {
		bytes = val
	})
	return bytesToFileData(bytes)
}

func (db *DB) SaveMinFileNumberToUpload(number uint64) error {
	return (*badger.DB)(db).Update(func(txn *badger.Txn) error {
		if badger.NewEntry("m", [8]byte(number)) == nil {
			return errors.New("cannot set min file number entry")
		}
	})
}

func (db *DB) GetMinFileNumberToUpload() (uint64, error) {
	var bytes []byte
	err := (*badger.DB)(db).View(func(txn *badger.Txn) error {
		item, err := txn.Get([]byte("m"))
		if err != nil {
			return err
		}
		err = item.Value(func(val []byte) error {
			bytes = val
			if err != nil {
				return err
			}
		})
		if err != nil {
			return err
		}
	})
	if err != nil {
		return 0, err
	}
	if len(bytes) != 8 {
		return 0, errors.New("wrong size of the current file number")
	}
	return binary.BigEndian.Uint64(bytes), err
}
