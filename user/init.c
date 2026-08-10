#include "user.h"

int main() {
    for (int i = 0;; i++) {
        printf("calculating %d-th prime: \n", i);
        if(!fork()){
            // child
            char buff[20];
            itoa(i, buff);
            char* args[] = {buff};
            exec("prime", args);
        } else{
            //parent
            wait(0);
        }
    }
}
