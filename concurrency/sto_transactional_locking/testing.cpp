#include <random>
#include <functional>
#include <string_view>
#include <chrono>
#include <iostream>
#include <thread>

#include "TransactionManager.h"


struct test_item;


using item_list = std::vector<test_item>;
using index_list = std::vector<size_t>;

using entropy = std::random_device;
using prn_gen = std::mt19937_64;
using int_dist = std::uniform_int_distribution<>;
using hasher = std::hash<std::string_view>;

//using chrono = std::chrono;


struct test_item : public lockable_item
{
    static constexpr size_t buf_size = 32;

    char ma_chars[buf_size]; // to be updated

    void singlethreaded_update(prn_gen& gen, int_dist& char_dist);
    void transactional_update(TransactionManager const& tx, prn_gen& gen, int_dist& char_dist);
};

void test_item::singlethreaded_update(prn_gen& gen, int_dist& char_dist)
{
    char local_chars[buf_size];
    std::string_view local_view(local_chars, buf_size);
    std::string_view shared_view(ma_chars, buf_size);
    hasher hash;

    for (size_t i = 0; i< buf_size; ++i)
    {
        local_chars[i] = this->ma_chars[i] = (char) char_dist(gen);
    }

    if (hash(shared_view) != hash(local_view))
    {
            std::cout << "ST RACE FOUND! " << this->id() << std::endl;
    }
}


void test_item::transactional_update(TransactionManager const& tx, prn_gen& gen, int_dist& char_dist)
{
    char local_chars[buf_size];
    std::string_view local_view(local_chars, buf_size);
    std::string_view shared_view(this->ma_chars, buf_size);
    hasher hash;

    for (size_t i = 0; i< buf_size; ++i)
    {
        local_chars[i] = this->ma_chars[i] = (char) char_dist(gen);
    }

    if (hash(shared_view) != hash(local_view))
    {
            std::cout << "TX RACE FOUND! ids: " << tx.id() << " " << this->id() << std::endl;
    }
}

void tx_access_test(item_list& items, size_t tx_count, size_t refs_count)
{
    entropy rd;
    prn_gen gen(rd());
    int_dist refs_index_dist(0, (int)(items.size() - 1));
    int_dist refs_count_dist(1, (int) refs_count);
    int_dist char_dist(0, 127);

    //chrono::stopwatch sw;
    index_list indices;
    size_t index;

    TransactionManager tx(1);
    bool acquired;

    //sw.start();

    for (size_t i=0; i<tx_count; ++i)
    {
        //compute the size of the update group
        indices.clear();
        refs_count = refs_count_dist(gen);

        // compute the membership of the update group
        for (size_t j =0; j< refs_count; ++j)
        {
            index = refs_index_dist(gen);
            indices.push_back(index);
        }

        tx.begin();
        acquired = true;

        for (size_t j =0; acquired && j < refs_count; ++j)
        {
            index = indices[j];
            acquired = tx.acquire(items[index]);
        }

        if (acquired)
        {
            for (size_t j=0; j < refs_count; ++j)
            {
                index = indices[j];
                items[index].transactional_update(tx, gen, char_dist);
            }
            tx.commit();
        }
        else
        {
            tx.rollback();
        }

    }
}

int main()
{
    constexpr size_t item_num = 10000, transactions = 100, refs = 50, thread_num = 4;
    std::cout << "starting threads: " << thread_num << " items: " << item_num << " transactions: " <<transactions << " refs: " << refs << std::endl;
    item_list items(item_num);

    auto doWork = [&]() { tx_access_test(items, transactions, refs);};

    std::array<std::thread, thread_num> threads;
    for (auto&& t : threads) {
        t = std::thread(doWork);
    }
    std::cout << "started" << std::endl;
    for (auto&& t : threads) {
        t.join();
    }
    std::cout << "joined" << std::endl;

}
