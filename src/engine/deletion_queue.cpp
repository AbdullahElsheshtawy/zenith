#include "deletion_queue.hpp"

void DeletionQueue::Push(std::function<void()> &&function) {
  Deletors_.emplace_back(function);
}

void DeletionQueue::Flush() {
  for (auto it = Deletors_.rbegin(); it != Deletors_.rend(); it++) {
    (*it)();
  }
  Deletors_.clear();
}
