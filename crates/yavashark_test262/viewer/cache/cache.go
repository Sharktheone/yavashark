package cache

import (
	"crypto/sha256"
	"encoding/json"
	"fmt"
	"os"
	"viewer/conf"
	"yavashark_test262_runner/results"
)

const ResultsPath = "results.json"

type CacheItem struct {
	Checksum       string
	results        []results.Result
	IndexedResults map[string]results.Result
	ciResults      []results.CIResult
}

var Cache = make(map[string]*CacheItem)

func InitWithCurrent() error {
	_, err := GetItem()
	return err
}

func GetItem() (*CacheItem, error) {
	sum, out, err := checksumFile(ResultsPath)

	if err != nil {
		return nil, err
	}

	cksum := fmt.Sprintf("%x", sum)

	if item, ok := Cache[cksum]; ok {
		return item, nil
	}

	var res []results.Result

	if err := json.Unmarshal(out, &res); err != nil {
		return nil, err
	}

	ciResults := results.ConvertResultsToCI(res, conf.TestRoot)

	item := &CacheItem{
		Checksum:       cksum,
		results:        res,
		ciResults:      ciResults,
		IndexedResults: indexResults(&res),
	}

	Cache[cksum] = item

	return item, nil
}

func GetCi() (*[]results.CIResult, error) {
	item, err := GetItem()
	if err != nil {
		return nil, err
	}

	return &item.ciResults, nil
}

func GetResults() (*[]results.Result, error) {
	item, err := GetItem()
	if err != nil {
		return nil, err
	}

	return &item.results, nil
}

func GetResultsIndex() (*map[string]results.Result, error) {
	item, err := GetItem()
	if err != nil {
		return nil, err
	}

	return &item.IndexedResults, nil
}

func checksumFile(path string) ([]byte, []byte, error) {
	f, err := os.ReadFile(path)
	if err != nil {
		//TODO: handle not found differently
		return nil, nil, err
	}

	h := sha256.New()
	if _, err := h.Write(f); err != nil {
		return nil, f, err
	}

	return h.Sum(nil), f, nil
}

func indexResults(res *[]results.Result) map[string]results.Result {
	indexedResults := make(map[string]results.Result, len(*res))

	for _, res := range *res {
		indexedResults[res.Path] = res
	}

	return indexedResults
}
