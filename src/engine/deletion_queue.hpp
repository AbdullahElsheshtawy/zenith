#pragma once
#include <vector>
#include <functional>

class DeletionQueue {
  std::vector<std::function<void()>> Deletors_;

public:
  void Push(std::function<void()>&& function);
  void Flush();
};