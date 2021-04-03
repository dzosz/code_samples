#include "TransactionManager.h"

#include <iostream>

TransactionManager::atomic_tsv TransactionManager::sm_tsv_generator{0};
TransactionManager::atomic_tx_id TransactionManager::sm_tx_id_generator{0};

TransactionManager::TransactionManager(int log_level) : m_tx_id(++sm_tx_id_generator), m_log_Level(log_level)
{
m_item_ptrs.reserve(100u);
}

void TransactionManager::begin()
{
    m_mutex.lock();
    m_tx_tsv = ++sm_tsv_generator;
    m_mutex.unlock();
}


void TransactionManager::commit()
{
    tx_lock lock(m_mutex);
    while (m_item_ptrs.size() !=0)
    {
        m_item_ptrs.back()->mp_owning_tx.store(nullptr);
        m_item_ptrs.pop_back();
    }
    m_cond.notify_all();
}

void TransactionManager::rollback()
{
    std::cout << "rollback: " << m_tx_tsv << std::endl;
    this->commit();
}

bool TransactionManager::acquire(lockable_item& item)
{

    while (true)
    {
        TransactionManager* p_curr_tx = nullptr;

        bool gotOwnership = item.mp_owning_tx.compare_exchange_strong(p_curr_tx, this);
        if (gotOwnership)
        {
            m_item_ptrs.push_back(&item);
            if (m_tx_tsv > item.m_last_tsv)
            {
                item.m_last_tsv = m_tx_tsv;
                return true;
            }
            else
            {
                // " a younger transaction as updated it"
                return false;
            }
        }
        else
        {
            if (p_curr_tx == this)
            {
                return true; // We already own this item

            }
            else
            {
                // wait
                tx_lock lock(p_curr_tx->m_mutex);
                while (item.mp_owning_tx.load() == p_curr_tx)
                {
                    if (p_curr_tx->m_tx_tsv > m_tx_tsv)
                    {
                        // too old
                        return false;
                    }
                    p_curr_tx->m_cond.wait(lock);
                    // go back to the top and retry
                }
            }
        }
    }
}

tx_id_type TransactionManager::id() const { return m_tx_id; }
tsv_type TransactionManager::tsv() const { return m_tx_tsv;}
