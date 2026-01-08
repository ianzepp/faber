// caelum.hpp - HTTP helper library for C++ target (stub)
//
// NOT YET IMPLEMENTED - requires libcurl or cpp-httplib

#pragma once

#include <stdexcept>
#include <string>
#include <map>

namespace caelum {

[[noreturn]] inline void pete(const std::string& url) {
    (void)url;
    throw std::runtime_error("caelum::pete not yet implemented for C++ target");
}

[[noreturn]] inline void mitte(const std::string& url, const std::string& corpus) {
    (void)url;
    (void)corpus;
    throw std::runtime_error("caelum::mitte not yet implemented for C++ target");
}

[[noreturn]] inline void pone(const std::string& url, const std::string& corpus) {
    (void)url;
    (void)corpus;
    throw std::runtime_error("caelum::pone not yet implemented for C++ target");
}

[[noreturn]] inline void dele(const std::string& url) {
    (void)url;
    throw std::runtime_error("caelum::dele not yet implemented for C++ target");
}

[[noreturn]] inline void muta(const std::string& url, const std::string& corpus) {
    (void)url;
    (void)corpus;
    throw std::runtime_error("caelum::muta not yet implemented for C++ target");
}

[[noreturn]] inline void roga(
    const std::string& modus,
    const std::string& url,
    const std::map<std::string, std::string>& capita,
    const std::string& corpus
) {
    (void)modus;
    (void)url;
    (void)capita;
    (void)corpus;
    throw std::runtime_error("caelum::roga not yet implemented for C++ target");
}

template<typename Handler>
[[noreturn]] inline void exspecta(Handler handler, int64_t portus) {
    (void)handler;
    (void)portus;
    throw std::runtime_error("caelum::exspecta not yet implemented for C++ target");
}

[[noreturn]] inline void replicatio(
    int64_t status,
    const std::map<std::string, std::string>& capita,
    const std::string& corpus
) {
    (void)status;
    (void)capita;
    (void)corpus;
    throw std::runtime_error("caelum::replicatio not yet implemented for C++ target");
}

} // namespace caelum
