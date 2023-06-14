#pragma once

#include <array>
#include <vector>
#include <limits>
#include <optional>
#include <cassert>
#include <cstdint>
#include <cstring>
#include <type_traits>
#include <iostream>
#include <xmmintrin.h>
#include <arpa/inet.h>


// assuming PACKED struct
struct __attribute__ ((__packed__)) Packet {
        uint32_t src; 
        uint32_t dst;
        uint8_t _14_proto;
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


// use gcc's vector_extensions for simd
// divide into four unsigned integers
// we can ignore fourth one, because the data is packed
// third element has to be masked because 
typedef int32_t DataVec __attribute__ ((vector_size (16), aligned (16)));

// alignment required for simd
struct __attribute__ ((__packed__,aligned(16))) CompressedRule {
    // simd this part 2*32+8+2*16=104
    uint32_t src_mask=std::numeric_limits<uint32_t>::min(); // 1st position in vector_size
    uint32_t dst_mask=std::numeric_limits<uint32_t>::min(); // 2nd position
    uint8_t _14_proto=std::numeric_limits<uint8_t>::max();  // 3rd position
    uint16_t sport_mask=std::numeric_limits<uint16_t>::max();//3rd position
    uint16_t dport_mask=std::numeric_limits<uint16_t>::max(); // 8 bits out of 16 belong to 4rd vector position
    // end simd
    //uint8_t padding[3]={}; // payload here

    uint32_t src=std::numeric_limits<uint32_t>::min();
    uint32_t dst=std::numeric_limits<uint32_t>::min();

    CompressedRule() = default;
    CompressedRule(const Rule& r) {
        if (r.src)
        {
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

    bool matchesSimd(const Packet& p) const;
};

bool CompressedRule::matchesSimd(const Packet& p) const {
    DataVec packet_aligned;
    memcpy(&packet_aligned, &p, sizeof(DataVec)); // packet is unaligned, so copy
    const DataVec* rule2 = reinterpret_cast<const DataVec*>(this);
    const DataVec allowed = packet_aligned & *rule2;
    
    packet_aligned[0] = src; // TODO if we could somehow get rid of this
    packet_aligned[1] = dst;

    const DataVec diff = packet_aligned ^ allowed; 
    return diff[0] == 0 && diff[1] == 0 && diff[2] == 0 && ((diff[3] >> 24) == 0); // last index contains some of the paylod so we need to shift
}


class Filter {
    public: 
        Filter(std::vector<Rule> rules);
        bool process(const Packet&) const;
    private:
        bool processVectorized(const Packet&) const;

      size_t num_rules;
      std::array<CompressedRule, 64> rules;
};

Filter::Filter(std::vector<Rule> copied_rules) {
    assert(copied_rules.size() <= 64);
    this->num_rules = copied_rules.size();
    for (size_t i=0; i < copied_rules.size(); ++i) {
        this->rules[i] = CompressedRule(copied_rules[i]);
    }
}

bool Filter::process(const Packet& packet) const { 
    return processVectorized(packet);
}
bool Filter::processVectorized(const Packet& p) const {
    size_t i = 0;
    // loop unrolling, this might not be helping that much in current state because ::matchesSimd() is still not small enough
    for (i =0; i+4 < this->num_rules ; i+=4) {
        if (rules[i].matchesSimd(p)) {
            return true;
        }
        if (rules[i+1].matchesSimd(p)) {
            return true;
        }
        if (rules[i+2].matchesSimd(p)) {
            return true;
        }
        if (rules[i+3].matchesSimd(p)) {
            return true;
        }
    }
    // leftover
    for (; i < this->num_rules ; i+=1) {
        if (rules[i].matchesSimd(p)) {
            return true;
        }
    }
    return false;
}

