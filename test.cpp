#include <iostream>
#include <vector>
#include <cmath>

int main() {
    std::vector<std::string> grid = {"hello", "world"};
    
    for(auto row : grid) std::cout << row << std::endl; 
    
    long ans = pow(2, 40);
    
    loop_start:
    ans--;
    if (ans > 0) {
        goto loop_start; 
    }
    
    return 0;
}