#pragma once

#include <array>
#include <vector>
#include <limits>
#include <optional>
#include <cassert>
#include <cstdint>
#include <type_traits>
#include <iostream>
#include <arpa/inet.h> // inet_addr

#include <bits/floatn-common.h>
#include <immintrin.h> // simd
#include <xmmintrin.h>

// not really modern but guarantees clean release build
#define LOGGING 0
template<class ...Args>
void log_dbg(Args&&... args) {
#if LOGGING > 0 
    do { ((std::cout << args << ' '), ...) << '\n'; } while(0);
#endif
}

std::ostream& operator<<(std::ostream& o, const __int128& x) {
    if (x == std::numeric_limits<__int128>::min()) return o << "-170141183460469231731687303715884105728";
    if (x < 0) return o << "-" << -x;
    if (x < 10) return o << (char)(x + '0');
    return o << x / 10 << (char)(x % 10 + '0');
}

struct __attribute__ ((__packed__)) Packet {
    uint32_t src;
    uint32_t dst;

    uint8_t _14_proto;
    //uint8_t padding

    uint16_t sport;
    uint16_t dport;
    uint8_t payload[1500];
};

struct Rule {
    struct Net {
        uint32_t addr;
        uint8_t bits;
    };
    std::optional<Net> src;
    std::optional<Net> dst;

    std::optional<uint8_t> _14_proto;

    std::optional<uint16_t> sport;
    std::optional<uint16_t> dport;
};


struct __attribute__ ((__packed__,aligned(16))) CompressedRule {
    // simd this part 2*32+8+2*16=104
    uint32_t src_mask=std::numeric_limits<uint32_t>::min();;
    uint32_t dst_mask=std::numeric_limits<uint32_t>::min();;
    uint8_t _14_proto=std::numeric_limits<uint8_t>::max();;
    uint16_t sport_mask=std::numeric_limits<uint16_t>::max();;
    uint16_t dport_mask=std::numeric_limits<uint16_t>::max();;
    // end simd
    uint32_t padding[2];

    uint32_t src=std::numeric_limits<uint32_t>::min();
    uint32_t dst=std::numeric_limits<uint32_t>::min();

    CompressedRule() = default;

    CompressedRule(const Rule& r) {
        if (r.src) {
            auto bits_move = (sizeof(uint32_t)*8-r.src->bits);
            uint32_t subnet = htonl(std::numeric_limits<uint32_t>::max() << bits_move);
            src = (r.src->addr & subnet);
            src_mask = subnet;
        }
        if (r.dst) {
            auto bits_move = (sizeof(uint32_t)*8-r.dst->bits);
            uint32_t subnet = htonl(std::numeric_limits<uint32_t>::max() << bits_move);
            dst = (r.dst->addr & subnet);
            dst_mask = subnet;
        }
        if (r._14_proto) {
            _14_proto = *r._14_proto;
        }
        if (r.sport) {
            sport_mask = *r.sport;
        }
        if (r.dport) {
            dport_mask = *r.dport;
        }
    }

    bool matches(const Packet& p) const;
    bool matchesSimd(const Packet& p) const;
    bool matchesSimd2(const Packet& p) const;
};

// TODO 1. unaligned load vs aligned load load_ vs loadu_
// compare32bit integers __mmask16 _mm512_cmpeq_epi32_mask (__m512i a, __m512i b)
bool CompressedRule::matchesSimd2(const Packet& p) const {
    const __m128i pack = _mm_loadu_si128(reinterpret_cast<const __m128i*>(&p)); 
    const __m128i rule = _mm_load_si128(reinterpret_cast<const __m128i*>(this));
    const __m128i mask = _mm_and_si128(pack, rule);
/*
    log_dbg("m", masked[0]);
    // TODO mmalignr_epi8 saves some instructions?
    // const __m128i next = _mm_alignr_epi8(chunk1, chunk0, 4);
    //const __m128i mask = _mm_cmpgt_epi32(curr, next);/
    //log_dbg("m", (uint8_t)mask);
    //log_dbg("p", (uint8_t)pack);
    //log_dbg("r", (uint8_t)rule);
}
    /*
    auto same_src = (p.src & src_mask) == src;
    auto same_dst = (p.dst & dst_mask) == dst;
    auto same_proto = (p._14_proto & _14_proto) == p._14_proto;
    auto same_sport = (p.sport & sport_mask) == p.sport;
    auto same_dport = (p.dport & dport_mask) == p.dport;
    */
    return false;
}

bool CompressedRule::matchesSimd(const Packet& p) const {
    //typedef int v8si __attribute__ ((vector_size (16)));
    typedef u_int32_t v4si __attribute__ ((vector_size (16), , aligned (16)));
    __m128i pack = _mm_loadu_si128(reinterpret_cast<const __m128i*>(&p)); 
    const __m128i rule = _mm_load_si128(reinterpret_cast<const __m128i*>(this));
    //v4si pack2 = *reinterpret_cast<const v4si*>(&p);
    //const v4si rule2 = *reinterpret_cast<const v4si*>(this);
    v4si& pack2 = reinterpret_cast<v4si&>(pack);
    const v4si& rule2 = reinterpret_cast<const v4si&>(rule);
    //const v4si to_compare = {src, dst, p._14_proto, (p.sport << 16) | (p.dport)};
    //v4si to_compare = *pack2;
    //log_dbg("pack ", (*pack2)[0], (*pack2)[1], (*pack2)[2]);
    //log_dbg("rule ", (*rule2)[0], (*rule2)[1], (*rule2)[2]);
    //log_dbg("to_cmp ", to_compare[0], to_compare[1], to_compare[2]);
    const v4si masked = pack2 & rule2;

    pack2[0] = src;
    pack2[1] = dst;
    log_dbg("masked ", masked[0], masked[1], masked[2]);
    const v4si diff = pack2 == masked;
    //const v4si diff = pack2 ^ masked;
    log_dbg("diff ", diff[0], diff[1], diff[2]);
    return diff[0] == -1 && diff[1] == -1 && diff[2] == -1;
    //return (to_compare ^ masked);
}

bool CompressedRule::matches(const Packet& p) const {
    auto same_src = (p.src & src_mask) == src;
    auto same_dst = (p.dst & dst_mask) == dst;
    auto same_proto = (p._14_proto & _14_proto) == p._14_proto;
    auto same_sport = (p.sport & sport_mask) == p.sport;
    auto same_dport = (p.dport & dport_mask) == p.dport;
    log_dbg(same_src , same_dst , same_proto , same_sport , same_dport);
    log_dbg("");
    //matchesSimd(p);

    log_dbg("packet.src as int:", p.src);
    log_dbg("packet.src:", Rule::Net{p.src, 0});
    log_dbg("rule.src as int:", src);
    log_dbg("rule.src:", Rule::Net{src, 0});
    log_dbg("rule.src_mask:", Rule::Net{src_mask, 0});
    log_dbg("p.src&src_mask:", Rule::Net{p.src&src_mask, 0});
    //log_dbg();

/*
    log_dbg("packet.dst:", Rule::Net{p.dst, 0});
    log_dbg("rule.dst:", Rule::Net{dst, 0});
    log_dbg("rule.dst_mask:", Rule::Net{dst_mask, 0});
    log_dbg("p.dst&dst_mask:", Rule::Net{p.dst&dst_mask, 0});

    /*
    log_dbg("");
    log_dbg("packet  :", Rule::Net{, 0});
    log_dbg("subnet: ", Rule::Net{htonl(subnet), cidr}, " same=", is_in_subnet);
    log_dbg("allowed&sub: ", Rule::Net{(allowed & subnet), 0});
    log_dbg("packet &sub: ", Rule::Net{(packet & subnet), 0});
    */
    return matchesSimd(p);
    return same_src && same_dst && same_proto && same_sport && same_dport;
}

std::ostream& operator<<(std::ostream& os, const Rule::Net& net) {
    auto addr = ntohl(net.addr); // to cpu order
    os << (addr >> 24 & 0xFF) << "." << (addr >> 16 & 0xFF) << "."
       << (addr >> 8 & 0xFF) << "." << (addr & 0xFF);
    os << ":" << static_cast<unsigned>(net.bits);
    return os;
}

class Filter {
    public: 
        Filter(std::vector<Rule> rules);
        bool process(const Packet&) const;
        bool processSlow(const Packet&) const;
        bool processFast(const Packet&) const;
    private:
        bool processSlowRule(const Packet&, const Rule& rule) const;

      bool allowed_in_subnet(uint32_t allowed, uint8_t cidr, uint32_t packet) const;
      bool allowed_protocol(uint8_t allowed_protocol, uint8_t packet_protocol) const;
      bool allowed_port(uint16_t allowed_port, uint16_t packet_port) const;

      std::vector<Rule> rules2;
      size_t num_rules;
      std::array<CompressedRule, 64> rules;
};

Filter::Filter(std::vector<Rule> copied_rules) // : rules{std::move(copied_rules)} 
{
    // TODO use compressed rules
    assert(copied_rules.size() <= 64);
    this->num_rules = copied_rules.size();
    for (int i =0; i < copied_rules.size(); ++i) {
        this->rules[i] = CompressedRule(copied_rules[i]);
    }
    log_dbg("num rules", num_rules);
    this->rules2 = std::move(copied_rules);
}

bool Filter::process(const Packet& packet) const { 
    return processFast(packet);
    //return processSlow(packet);
}
bool Filter::processFast(const Packet& p) const {
    int i = 0;
    for (i =0; i+8 < this->num_rules ; i+=8) {
        if (rules[i].matches(p)) {
            return true;
        }
        if (rules[i+1].matches(p)) {
            return true;
        }
        if (rules[i+2].matches(p)) {
            return true;
        }
        if (rules[i+3].matches(p)) {
            return true;
        }
    }
    for (; i < this->num_rules ; i+=1) {
        if (rules[i].matches(p)) {
            return true;
        }
    }
    return false;
}

bool Filter::processSlowRule(const Packet& packet, const Rule& rule) const {
    if (rule.src && !this->allowed_in_subnet(rule.src->addr, rule.src->bits, packet.src)) {
        return false;
    }
    if (rule.dst && !this->allowed_in_subnet(rule.dst->addr, rule.dst->bits, packet.dst)) {
        return false;
    }

    if (rule._14_proto && !this->allowed_protocol(*rule._14_proto, packet._14_proto)) {
        return false;
    }

    if (rule.sport && !this->allowed_port(*rule.sport, packet.sport)) {
        return false;
    }

    if (rule.dport && !this->allowed_port(*rule.dport, packet.dport)) {
        return false;
    }
    return true; // the rule fits the packet
}
bool Filter::processSlow(const Packet& packet) const {
    /*
    for (auto& rule : this->rules2) {
    }
    return false;
    */

    int i = 0;
    for (i =0; i+4 < this->num_rules ; i+=4) {
        if (processSlowRule(packet, rules2[i])) {
            return true;
        }
        if (processSlowRule(packet, rules2[i+1])) {
            return true;
        }
        if (processSlowRule(packet, rules2[i+2])) {
            return true;
        }
        if (processSlowRule(packet, rules2[i+3])) {
            return true;
        }
    }
    // leftovers
    for (; i < this->num_rules ; i+=1) {
        if (processSlowRule(packet, rules2[i])) {
            return true;
        }
    }
    return false;
}

bool Filter::allowed_protocol(uint8_t allowed_protocol, uint8_t packet_protocol) const {
    return allowed_protocol == packet_protocol;
}

bool Filter::allowed_port(uint16_t allowed_port, uint16_t packet_port) const {
    return allowed_port == packet_port;
}

bool Filter::allowed_in_subnet(uint32_t allowed, uint8_t cidr, uint32_t packet) const {
    //cidr = ntohs(cidr);   // TODO cidr should be little endian
    auto bits_move = (sizeof(uint32_t)*8-cidr);
    uint32_t subnet = htonl(std::numeric_limits<uint32_t>::max() << bits_move);
    //uint32_t subnet = ~((1 << bits_move) - 1);
    //bool is_in_subnet = (allowed & subnet) == (packet & subnet);
    bool is_in_subnet = (allowed & subnet) == (packet & subnet);
    
    log_dbg("");
    log_dbg("rule.src:", Rule::Net{allowed, 0});
    log_dbg("packet  :", Rule::Net{packet, 0});
    log_dbg("subnet: ", Rule::Net{htonl(subnet), cidr}, " same=", is_in_subnet);
    log_dbg("allowed&sub: ", Rule::Net{(allowed & subnet), 0});
    log_dbg("packet &sub: ", Rule::Net{(packet & subnet), 0});

    return is_in_subnet;
}
