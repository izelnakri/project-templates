#ifndef USER_HPP
#define USER_HPP

#include <string>

/**
 * @brief Represents a GitHub user.
 */
class User {
public:
    /**
     * @brief Constructs a User with given properties.
     * 
     * @param login The user's login name.
     * @param name The user's full name.
     * @param company The user's company.
     * @param location The user's location.
     */
    User(std::string login, std::string name, std::string company, std::string location);

    const std::string& getLogin() const;
    const std::string& getName() const;
    const std::string& getCompany() const;
    const std::string& getLocation() const;

    /**
     * @brief Prints user information to stdout.
     */
    void print() const;

private:
    std::string login_;
    std::string name_;
    std::string company_;
    std::string location_;
};

/**
 * @brief Fetches GitHub user information for the given username.
 * 
 * Makes an HTTPS GET request to https://api.github.com/users/{username}.
 *
 * @param username The GitHub username.
 * @return User object with parsed user data.
 * @throws std::runtime_error if the request fails or the user does not exist.
 */
User fetch_github_user(const std::string& username);

#endif // USER_HPP
