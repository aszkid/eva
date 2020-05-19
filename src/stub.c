#include <stdarg.h>
#include <stdio.h>
#include <sys/socket.h>
#include <sys/un.h>

extern void eva_syslog(int priority, char* buf);

#define CLIENT_SOCK_FILE "eva_client.sock"
#define SERVER_SOCK_FILE "eva_server.sock"
static int fd = -1;

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
    
    printf("hi from C!\n");
    eva_syslog(priority, buf);

    struct sockaddr_un addr;
    struct sockaddr_un from;
    int ret;
    int len;

    if (fd == - 1 && (fd = socket(PF_UNIX, SOCK_DGRAM, 0)) < 0) {
        perror("socket");
        return;
    }

    memset(&addr, 0, sizeof(addr));
    addr.sun_family = AF_UNIX;
    strcpy(addr.sun_path, CLIENT_SOCK_FILE);
    unlink(CLIENT_SOCK_FILE);
    if (bind(fd, (struct sockaddr *)&addr, sizeof(addr)) < 0) {
        perror("bind");
        return;
    }

    memset(&addr, 0, sizeof(addr));
    addr.sun_family = AF_UNIX;
    strcpy(addr.sun_path, SERVER_SOCK_FILE);
    if (connect(fd, (struct sockaddr *)&addr, sizeof(addr)) == -1) {
        perror("connect");
        return;
    }

    if (send(fd, buf, strlen(buf) + 1, 0) == -1) {
        perror("send");
        return;
    }

    if (fd >= 0)
        close(fd);

    unlink(CLIENT_SOCK_FILE);
}

