#include <iostream>
#include <vector>
#include <cmath>
#define ll long long
#define vi std::vector<int>

using vll = std::vector<ll>;
int main() {
    std::vector<std::string> grid = {"hello", "world"};
    std::vector<bool> yeah;

    int n;
    vector<int> arr(n);
    
    for(auto row : grid) std::cout << row << std::endl; 
    
    long ans = pow(2, 40);

    
    if(ans & 1 == 0) std::cout << "Ans is odd :D" << std::endl;
    else std::cout << "Bruh Ans is even\n";
    
    loop_start:
    ans--;
    if (ans > 0) {
        goto loop_start; 
    }

    
    return 0;
}