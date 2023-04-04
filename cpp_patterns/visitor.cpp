#include <vector>
#include <memory>
#include <iostream>
#include <variant>

struct Circle;

struct ShapeVisitor {
    virtual void visit(const Circle&) const = 0;
};

struct Shape {
    virtual void accept(const ShapeVisitor&) const = 0;
    virtual ~Shape() = default;
};

struct Circle : public Shape {
    void accept(const ShapeVisitor& v) const {
        v.visit(*this);
    }
    ~Circle() = default;
};


struct Draw : public ShapeVisitor {
    virtual void visit(const Circle& c) const {
        std::cout << "Draw circle: " << &c << std::endl;
    }
};

void drawAllOOP(const std::vector<std::unique_ptr<Shape>>& shapes) {
    for (const auto& s : shapes) {
        // s->draw();
    }
}

void drawAllVisitor(const std::vector<std::unique_ptr<Shape>>& shapes) {
    for (const auto& s : shapes) {
        s->accept(Draw());
    }
}

struct DrawVariant {
    void operator()(const Circle& c) {
        std::cout << "DrawVariant circle: " << &c << std::endl;
    }
};

void drawAllVariants(const std::vector<std::variant<Circle>>& shapes) {
    DrawVariant drawer;
    for (const auto& s : shapes) {
        std::visit(drawer, s);
    }
}


int main() {
    {  // classic
        std::vector<std::unique_ptr<Shape>> collection;
        collection.emplace_back(new Circle());
        drawAllVisitor(collection);
    }
    {  // variant based
        std::vector<std::variant<Circle>> collection;
        collection.emplace_back(Circle{});
        drawAllVariants(collection);
    }
}
