#include "user.h"

int main() {
    for (int i = 1;; i++) {
        printf("calculating %d-th prime: \n", i);
        if(!fork()){
            // child
            char buff[20];
            memset(buff, 0, 20);
            itoa(i, buff);
            char* args[] = {buff, 0};
            exec("prime", args);
        } else{
            //parent
            wait(0);
        }
    }
}
