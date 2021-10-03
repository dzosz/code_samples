#define CATCH_CONFIG_MAIN
#define CATCH_CONFIG_ENABLE_BENCHMARKING
//#include "catch2_static/include/catch_with_main.hpp"
#include "catch2/catch.hpp"
#include "firewall.hpp"

#include <arpa/inet.h> // inet_addr
#include <random>

uint32_t ip_to_num(const char* ip) {
    return inet_addr(ip); // outputs network order - bigendian | TODO this function doesn't handle errors well (-1=255.255.255.255)
}

Packet get_example_packet() {
    Packet p;
    p.src = ip_to_num("100.50.194.0");
    p.dst = ip_to_num("100.50.194.0");
    p.sport = 130;
    p.dport = 60;
    p._14_proto = 16;
    p.payload[0] = -1;
    return p;
}
TEST_CASE( "24 subnet source", "[Rule]" ) {
    std::vector<Rule> rules;
    Rule r{};
    r.src = Rule::Net{ip_to_num("192.193.194.0"), 24};

    rules.push_back(r);

    Filter f(rules);

    auto p = get_example_packet();
    p.src = r.src->addr; // same
    CHECK( f.process(p) );

    p.src = ip_to_num("192.193.193.0"); // different
    CHECK( ! f.process(p) );
}
TEST_CASE( "empty rule ", "[Rule]" ) {
    std::vector<Rule> rules;
    Rule r;
    rules.push_back(r);

    Filter f(rules);

    auto p = get_example_packet();
    CHECK( f.process(p) );
}

TEST_CASE( " matching ports ", "[Rule]" ) {
    std::vector<Rule> rules;
    Rule r;
    r.sport = 100;
    r.dport = 100;
    rules.push_back(r);

    Filter f(rules);

    auto p = get_example_packet();
    p.dport = 100;
    p.sport = 100;
    CHECK( f.process(p) );
}

TEST_CASE( " mismatching ports ", "[Rule]" ) {
    std::vector<Rule> rules;
    Rule r;
    r.sport = 100;
    r.dport = 100;
    rules.push_back(r);

    Filter f(rules);

    auto p = get_example_packet();
    p.dport = 99;
    p.sport = 100;
    CHECK( ! f.process(p) );
}

TEST_CASE( " mismatching protocol ", "[Rule]" ) {
    std::vector<Rule> rules;
    Rule r;
    r._14_proto = 30;
    rules.push_back(r);

    Filter f(rules);

    auto p = get_example_packet();
    p._14_proto = 31;
    CHECK( ! f.process(p) );
}

TEST_CASE( " matching protocol ", "[Rule]" ) {
    std::vector<Rule> rules;
    Rule r;
    r._14_proto = 30;
    rules.push_back(r);

    Filter f(rules);

    auto p = get_example_packet();
    p._14_proto = 30;
    CHECK( f.process(p) );
}

TEST_CASE( "24 subnet destination", "[Rule]" ) {
    std::vector<Rule> rules;
    Rule r;
    r.dst = Rule::Net{ip_to_num("192.168.20.32"), 24};
    rules.push_back(r);

    Filter f(rules);

    Packet p;

    // allow
    p.dst = ip_to_num("192.168.20.31");
    REQUIRE( f.process(p) );
    p.dst = ip_to_num("192.168.20.32");
    REQUIRE( f.process(p) );
    p.dst = ip_to_num("192.168.20.33");
    REQUIRE( f.process(p) );
    p.dst = ip_to_num("192.168.20.34");
    REQUIRE( f.process(p) );
    p.dst = ip_to_num("192.168.20.35");
    REQUIRE( f.process(p) );
    p.dst = ip_to_num("192.168.20.36");
    REQUIRE( f.process(p) );
    p.dst = ip_to_num("192.168.20.254");
    REQUIRE( f.process(p) );
    p.dst = ip_to_num("192.168.20.157");
    REQUIRE( f.process(p) );

    // disallow
    p.dst = ip_to_num("192.168.21.1");
    REQUIRE( ! f.process(p) );
    p.dst = ip_to_num("192.168.19.255");
    REQUIRE( ! f.process(p) );
    p.dst = ip_to_num("192.168.24.1");
    REQUIRE( ! f.process(p) );
}

TEST_CASE( "23 subnet destination", "[Rule]" ) {
    std::vector<Rule> rules;
    Rule r;
    r.dst = Rule::Net{ip_to_num("192.168.20.32"), 23};
    rules.push_back(r);

    Filter f(rules);

    Packet p;

    // allow
    p.dst = ip_to_num("192.168.20.31");
    REQUIRE( f.process(p) );
    p.dst = ip_to_num("192.168.20.32");
    REQUIRE( f.process(p) );
    p.dst = ip_to_num("192.168.20.33");
    REQUIRE( f.process(p) );
    p.dst = ip_to_num("192.168.20.254");
    REQUIRE( f.process(p) );
    p.dst = ip_to_num("192.168.21.254");
    REQUIRE( f.process(p) );

    p.dst = ip_to_num("192.168.19.255");
    REQUIRE( ! f.process(p) );
    p.dst = ip_to_num("192.168.24.1");
    REQUIRE( ! f.process(p) );
}

TEST_CASE( "31 subnet destination", "[Rule]" ) {
    std::vector<Rule> rules;
    Rule r;
    r.dst = Rule::Net{ip_to_num("192.168.20.32"), 31}; // TODO rule should be little endian
    rules.push_back(r);

    Filter f(rules);

    Packet p;

    std::cout << std::hex;
    // allow
    p.dst = ip_to_num("192.168.20.32");
    CHECK( f.process(p) );

    p.dst = ip_to_num("192.168.20.33");
    CHECK( f.process(p) );

    // disallow
    p.dst = ip_to_num("192.168.20.34");
    REQUIRE( ! f.process(p) );
    p.dst = ip_to_num("192.168.20.35");
    REQUIRE( ! f.process(p) );
    p.dst = ip_to_num("192.168.20.36");
    REQUIRE( ! f.process(p) );
    p.dst = ip_to_num("192.168.20.254");
    REQUIRE( ! f.process(p) );
    p.dst = ip_to_num("192.168.20.157");
    REQUIRE( ! f.process(p) );

    p.dst = ip_to_num("192.168.21.1");
    REQUIRE( ! f.process(p) );
    p.dst = ip_to_num("192.168.19.255");
    REQUIRE( ! f.process(p) );
    p.dst = ip_to_num("192.168.24.1");
    REQUIRE( ! f.process(p) );
    p.dst = ip_to_num("192.168.21.1");
    REQUIRE( ! f.process(p) );
    p.dst = ip_to_num("192.168.20.31");
    REQUIRE( ! f.process(p) );
}

const static int SEED_FOR_TESTS=0;
static std::default_random_engine eng(SEED_FOR_TESTS);

static std::uniform_int_distribution<uint32_t> dist(0, std::numeric_limits<uint32_t>::max());

std::vector<Rule> generate_random_rules() {
    std::vector<Rule> rules;
    
    //auto r = rand() % 2;
    for (int i =0; i < 60; ++i ) {
        Rule r;
        r.dst = Rule::Net{(uint32_t)rand(), (uint8_t)24 + (uint8_t)(dist(eng)%6)};
        r.src = Rule::Net{(uint32_t)rand(), (uint8_t)24 + (uint8_t)(dist(eng)%6)};
        r.dport = (uint16_t)rand();
        r.sport = (uint16_t)rand();
        r._14_proto = (uint8_t)rand();
        rules.push_back(r);
    }

    return rules;
}

std::vector<Packet> generate_random_packets(size_t num_packets) {
    std::vector<Packet> packets;
    
    //auto r = rand() % 2;
    for (size_t i =0; i < num_packets; ++i ) {
        Packet r;
        r.dst = (uint32_t)rand();
        r.src = (uint32_t)rand();
        r.dport = (uint16_t)rand();
        r.sport = (uint16_t)rand();
        r._14_proto = (uint8_t)rand();
        packets.push_back(r);
    }

    return packets;
}


TEST_CASE( "bench_warmup_ignore ") {
    BENCHMARK_ADVANCED("advanced") (Catch::Benchmark::Chronometer meter) {
        auto rules = generate_random_rules();
        size_t max_packets = meter.runs();
        auto packets = generate_random_packets(max_packets);
        Filter f(rules);
        meter.measure([&packets, &f, max_packets](size_t i) { return f.processSlow(packets[i]); });
    };
}
TEST_CASE( "bench_fast") {
    BENCHMARK_ADVANCED("advanced") (Catch::Benchmark::Chronometer meter) {
        auto rules = generate_random_rules();
        size_t max_packets = meter.runs();
        auto packets = generate_random_packets(max_packets);
        Filter f(rules);
        meter.measure([&packets, &f, max_packets](size_t i) { return f.processFast(packets[i]); });
    };
}
TEST_CASE( "bench_slow") {
    BENCHMARK_ADVANCED("advanced") (Catch::Benchmark::Chronometer meter) {
        auto rules = generate_random_rules();
        size_t max_packets = meter.runs();
        auto packets = generate_random_packets(max_packets);
        Filter f(rules);
        meter.measure([&packets, &f, max_packets](size_t i) { return f.processSlow(packets[i]); });
    };
}
