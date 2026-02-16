#!/bin/bash
# Quick start script for Sudatta's frontend

echo "🚀 Setting up Sudatta's E-commerce Frontend..."
echo ""

cd "$(dirname "$0")"

# Check if node_modules exists
if [ ! -d "node_modules" ]; then
    echo "📦 Installing dependencies..."
    npm install
    echo "✅ Dependencies installed!"
else
    echo "✅ Dependencies already installed"
fi

echo ""
echo "📝 Next steps:"
echo ""
echo "1. Copy the storefront component:"
echo "   cp ../design.txt src/pages/Storefront.js"
echo ""
echo "2. Update src/App.js to use the Storefront component"
echo ""
echo "3. Start the development server:"
echo "   npm start"
echo ""
echo "4. Open http://localhost:3000 in your browser"
echo ""
echo "📚 Read IMPLEMENTATION_GUIDE.md for detailed instructions"
echo ""
