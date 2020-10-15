#ifndef LIBAKAZA_TMPFILE_H
#define LIBAKAZA_TMPFILE_H

#include <string>
#include <cstring>
#include <unistd.h>
#include <filesystem>

class TmpFile {
    std::string _path;
public:
    TmpFile() {
        char *name = strdup("/akazatest-XXXXXX");
        mkstemp(name);
        this->_path = std::filesystem::temp_directory_path().concat(name).string();
        free(name);
    }
    ~TmpFile() {
        unlink(this->_path.c_str());
    }

    std::string get_name() {
        return _path;
    }
};

#endif //LIBAKAZA_TMPFILE_H
