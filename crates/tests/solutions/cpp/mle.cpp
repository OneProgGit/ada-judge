#include <iostream>
#include <vector>

int main() {
    std::vector<int> big_vec(179'179'179, 179);
    std::cout << big_vec[179];
}