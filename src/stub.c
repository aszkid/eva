#include <stdarg.h>
#include <stdio.h>
#include <sys/socket.h>
#include <sys/un.h>
#include <unistd.h>
#include <stddef.h>
#include <stdlib.h>
#include <time.h>
#include <stdbool.h>
#include <arpa/inet.h>

#define SERVER_SOCK_FILE "/home/aszkid/dev/eva/eva_server.sock"
int fd = -1;


void _eva_send(char *payload, bool first)
{
    long int tstamp = htonl(time(NULL));

    // send timestamp
    if (!first) {
        if (send(fd, &tstamp, sizeof(long int), 0) == -1) {
            perror("send");
            return;
        }
    }

    // send payload
    if (send(fd, payload, strlen(payload) + 1, 0) == -1) {
        perror("send");
        return;
    }
}

/*
 * Connect to eva UNIX socket.
 */
void _eva_openlog()
{
    char buf[512];
    struct sockaddr_un addr;
    int len;

    if (fd >= 0)
        return;

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
    
    _eva_send(getenv("EVA_SERVICE"), true);
}

/*
 * Close socket connection.
 */
void _eva_closelog()
{
    close(fd);
}

//////////////////////////////////////////////////////////
// syslog API

void openlog(const char *ident, int option, int facility)
{
    _eva_openlog();
}

void closelog(void)
{
    _eva_closelog();
}

void syslog(int priority, char* fmt, ...)
{
    va_list args1;
    va_start(args1, fmt);
    va_list args2;
    va_copy(args2, args1);
    char payload[1+vsnprintf(NULL, 0, fmt, args1)];
    va_end(args1);
    vsnprintf(payload, sizeof payload, fmt, args2);
    va_end(args2);

    _eva_openlog();

    _eva_send(payload, false);
}

//////////////////////////////////////////////////////////