#include <stdio.h>

int main() {
    unsigned int state = 0x5b7081c3;
    int EzBkLX = 0;
    int MJkcnT = 0;
    int IWMhki = 0;
    while (state != 0x3161a7b5) {
        switch(state) {
            case 0x5b7081c3:
            {
                EzBkLX = 100;
                state = 0x73de7000;
                break;
            }
            case 0x42ac3f79:
            {
                unsigned char WQEFSN[] = {0x5a, 0x77, 0x73, 0x60, 0x66, 0x76, 0x74, 0x61, 0x7c, 0x7a, 0x7b, 0x35, 0x74, 0x76, 0x61, 0x7c, 0x63, 0x74, 0x61, 0x70, 0x71, 0x15};
                for(int i=0; i<22; i++) WQEFSN[i] ^= 0x15;
                printf("%s\n", WQEFSN);
                state = 0x41479947;
                break;
            }
            case 0x73de7000:
            {
                int bHjsCz = 0xf3ba;
                MJkcnT = 5;
                int GmbGQk = 0x4d;
                state = 0x6a65cce0;
                break;
            }
            case 0x41479947:
            {
                printf("%d\n", IWMhki);
                state = 0x3161a7b5;
                break;
            }
            case 0x6a65cce0:
            {
                IWMhki = EzBkLX + MJkcnT;
                state = 0x42ac3f79;
                break;
            }
            default:
                state = 0x3161a7b5;
                break;
        }
    }
    return 0;
}

