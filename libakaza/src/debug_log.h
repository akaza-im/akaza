//
// Created by tokuhirom on 10/9/20.
//

#ifndef LIBAKAZA_DEBUG_LOG_H
#define LIBAKAZA_DEBUG_LOG_H

#if 1
#define D(x) do {  } while (0)
#else
#define D(x) do { (x); } while (0)
#endif

#endif //LIBAKAZA_DEBUG_LOG_H
