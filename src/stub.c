#include <stdarg.h>
#include <stdio.h>
#include <sys/socket.h>
#include <sys/un.h>
#include <unistd.h>
#include <stddef.h>
#include <stdlib.h>

int fd = -1;

void openlog(const char *ident, int option, int facility)
{
    // nothing for now
}

void closelog(void)
{
    close(fd);
}

void syslog(int priority, char* fmt, ...)
{
    va_list args1;
    va_start(args1, fmt);
    va_list args2;
    va_copy(args2, args1);
    char buf[1+vsnprintf(NULL, 0, fmt, args1)];
    va_end(args1);
    vsnprintf(buf, sizeof buf, fmt, args2);
    va_end(args2);

    char SERVER_SOCK_FILE[512];
    sprintf(SERVER_SOCK_FILE, "/home/aszkid/dev/eva/eva_server.sock");
    
    char final_buf[512];
    struct sockaddr_un addr;
    int len;

    // create socket if needed
    if (fd < 0) {
        if ((fd = socket(AF_UNIX, SOCK_STREAM, 0)) < 0) {
            perror("socket");
            return;
        }

        // initialize and connect to eva server
        memset(&addr, 0, sizeof(addr));
        addr.sun_family = AF_UNIX;
        strcpy(addr.sun_path, SERVER_SOCK_FILE);
        len = (offsetof (struct sockaddr_un, sun_path)
            + strlen(addr.sun_path) + 1);

        if (connect(fd, (struct sockaddr *)&addr, len) == -1) {
            perror("connect");
            return;
        }
        
        sprintf(final_buf, "%s\n", getenv("EVA_SERVICE"));
        if (send(fd, final_buf, strlen(final_buf) + 1, 0) == -1) {
            perror("send");
            return;
        }
    }

    sprintf(final_buf, "%s\n", buf);
    if (send(fd, final_buf, strlen(final_buf) + 1, 0) == -1) {
        perror("send");
        return;
    }
}

