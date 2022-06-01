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
	uploaded := byte(0)
	if data.Uploaded {
		uploaded = 1
	}
	var result []byte
	result = append(data.Hash[:], uploaded)
	result = append(result, []byte(data.Name)...)
	return result
}

func bytesToFileData(bytes []byte) (*FileData, error) {
	if len(bytes) <= 33 {
		return nil, errors.New("too short record")
	}
	return &FileData{Hash: *(*[32]byte)(bytes[:32]), Uploaded: bytes[32] != 0, Name: string(bytes[33:])}, nil
}

type DB badger.DB

var ErrKeyNotFound = badger.ErrKeyNotFound

func OpenDB(fileName string) (*DB, error) {
	db, err := badger.Open(badger.DefaultOptions(fileName))
	if err != nil {
		return nil, err
	}
	return (*DB)(db), nil
}

func (db *DB) CloseDB() error {
	return (*badger.DB)(db).Close()
}

func (db *DB) SaveFileData(number uint64, data *FileData) error {
	return (*badger.DB)(db).Update(func(txn *badger.Txn) error {
		b := make([]byte, 8)
		binary.LittleEndian.PutUint64(b, number)
		if badger.NewEntry(b, fileDataToBytes(data)) == nil {
			return errors.New("cannot set DB entry")
		}
		return nil
	})
}

func (db *DB) ReadFileData(number uint64) (*FileData, error) {
	var item *badger.Item
	var err error
	err = (*badger.DB)(db).View(func(txn *badger.Txn) error {
		b := make([]byte, 8)
		binary.LittleEndian.PutUint64(b, number)

		item, err = txn.Get(b)
		return err
	})
	if err != nil {
		return nil, err
	}
	var bytes []byte
	item.Value(func(val []byte) error {
		bytes = val
		return nil
	})
	return bytesToFileData(bytes)
}

func (db *DB) SaveMinFileNumberToUpload(number uint64) error {
	return (*badger.DB)(db).Update(func(txn *badger.Txn) error {
		b := make([]byte, 8)
		binary.LittleEndian.PutUint64(b, number)
		if badger.NewEntry([]byte("m"), b) == nil {
			return errors.New("cannot set min file number entry")
		}
		return nil
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
			return nil
		})
		return err // needed?
	})
	if err != nil {
		return 0, err
	}
	if len(bytes) != 8 {
		return 0, errors.New("wrong size of the current file number")
	}
	return binary.LittleEndian.Uint64(bytes), err
}
