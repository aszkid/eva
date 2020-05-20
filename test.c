#include <syslog.h>
#include <stdio.h>

int main()
{
    syslog(LOG_USER | LOG_INFO, "!!! Hello world from syslog, %d", 32);
    syslog(LOG_USER | LOG_INFO, "Hi again: %s", "good lord");
    syslog(LOG_USER | LOG_INFO, "More log?");
    closelog();
    printf("good old printf\n");

    return 0;
}
