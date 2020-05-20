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
#include <stdint.h>

#if defined(__linux__)
#  include <endian.h>
#elif defined(__FreeBSD__) || defined(__NetBSD__)
#  include <sys/endian.h>
#elif defined(__OpenBSD__)
#  include <sys/types.h>
#  define be16toh(x) betoh16(x)
#  define be32toh(x) betoh32(x)
#  define be64toh(x) betoh64(x)
#endif

#define SERVER_SOCK_FILE "/home/aszkid/dev/eva/eva_server.sock"
int fd = -1;

int64_t time_nsec()
{
    struct timespec t;
	clock_gettime(CLOCK_REALTIME, &t);
    int64_t nsec = (int64_t)(t.tv_sec) * (int64_t)1e9 + (int64_t)(t.tv_nsec);
    return htobe64(nsec);
}

void _eva_send(char *payload, bool first)
{
    int64_t tstamp = time_nsec();

    // send timestamp
    if (!first) {
        if (send(fd, &tstamp, sizeof(int64_t), 0) == -1) {
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