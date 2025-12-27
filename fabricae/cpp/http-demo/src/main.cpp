#include <crow>
#include <cstdint>
#include <print>
#include <string>
#include <type_traits>
struct User {
    int64_t id = 0;
    std::string nomen = std::string("");
    std::string email = std::string("");
    bool active = true;

    User() = default;

    template<typename Overrides>
        requires std::is_aggregate_v<Overrides>
    User(const Overrides& o) {
        if constexpr (requires { o.id; }) id = o.id;
        if constexpr (requires { o.nomen; }) nomen = o.nomen;
        if constexpr (requires { o.email; }) email = o.email;
        if constexpr (requires { o.active; }) active = o.active;
    }
};
struct App {
    int64_t nextId = 1;

    App() = default;

    template<typename Overrides>
        requires std::is_aggregate_v<Overrides>
    App(const Overrides& o) {
        if constexpr (requires { o.nextId; }) nextId = o.nextId;
    }
};
bool isValidId(int64_t id) {
    return (id > 0);
}
bool validateUser(const User& user) {
    if (!isValidId(user.id)) {
        return false;
    }
    if ((user.nomen == std::string(""))) {
        return false;
    }
    return true;
}
std::any userToJson(const User& user) {
    return {.id = user.id, .nomen = user.nomen, .email = user.email, .active = user.active};
}
void handleIndex(const request& req, response& res) {
    res.body = std::string("Salve! C++ HTTP Demo");
}
void handleHealth(const request& req, response& res) {
    res.json({.status = std::string("ok")});
}
void handleGetUsers(const request& req, response& res) {
    res.json({.message = std::string("User list"), .count = 0});
}
void handleGetUser(const request& req, response& res, int64_t id) {
    if (!isValidId(id)) {
        res.code = 400;
        res.json({.error = std::string("Invalid ID")});
        return;
    }
    res.json({.id = id, .nomen = std::string("Marcus"), .email = std::string("marcus@roma.it")});
}
void handleCreateUser(const request& req, response& res, App& app) {
    const auto id = app.nextId;
    app.nextId += 1;
    res.code = 201;
    res.json({.id = id, .message = std::string("Created")});
}
void handleDeleteUser(const request& req, response& res, int64_t id) {
    res.code = 204;
}

int main() {

    std::print("{}\n", std::string("Initializing HTTP server..."));
    auto app = App{};
    auto server = SimpleApp{};
    server.route(std::string("/")).get([&](auto req, auto res) {
        handleIndex(req, res);
    });
    server.route(std::string("/health")).get([&](auto req, auto res) {
        handleHealth(req, res);
    });
    server.route(std::string("/users")).get([&](auto req, auto res) {
        handleGetUsers(req, res);
    });
    server.route(std::string("/users/<int>")).get([&](auto req, auto res, auto id) {
        handleGetUser(req, res, id);
    });
    server.route(std::string("/users")).post([&](auto req, auto res) {
        handleCreateUser(req, res, app);
    });
    server.route(std::string("/users/<int>")).delete([&](auto req, auto res, auto id) {
        handleDeleteUser(req, res, id);
    });
    std::print("{}\n", std::string("Server running on http://localhost:3000"));
    server.port(3000).run();
    return 0;
}