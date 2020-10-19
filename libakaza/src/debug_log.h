#ifndef LIBAKAZA_DEBUG_LOG_H
#define LIBAKAZA_DEBUG_LOG_H

#define DEBUG 0

#if DEBUG
#define D(x) do { (x); } while (0)
#else
#define D(x) do {  } while (0)
#endif

#endif //LIBAKAZA_DEBUG_LOG_H
