#include <iostream>
#include <thread>
#include <chrono>

int main() {
    int a, b;
    std::cin >> a >> b;
    std::this_thread::sleep_for(std::chrono::seconds(2));
    std::cout << a + b;
}