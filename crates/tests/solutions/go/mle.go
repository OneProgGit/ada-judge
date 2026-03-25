package main

import "fmt"

func main() {
	big_vec := make([]int, 179_179_179)

	for i := 0; i < len(big_vec); i += 179_79 {
		big_vec[i] = 179
	}

	fmt.Println(big_vec[179])
}