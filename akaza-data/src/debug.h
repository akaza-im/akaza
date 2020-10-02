static void dump_string(std::string buf) {
    const char * q=buf.c_str();
    for (int i=0; i<buf.size(); i++) {
        std::cout << +((uint8_t)q[i]) << " ";
    }
    std::cout << std::endl;
}

