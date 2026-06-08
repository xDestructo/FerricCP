#include <iostream>

int main() {
    int n;
    std::cin >> n;
    
    loop_start:
    n--;
    if(n > 0) {
        goto loop_start; 
    }
    
    return 0;
}