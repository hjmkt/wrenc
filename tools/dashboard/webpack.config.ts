import path from "path";
import webpack from "webpack";

const config: webpack.Configuration = {
    entry: "./src/index.tsx",
    module: {
        rules: [
            {
                test: /\.(ts|js)x?$/,
                exclude: /node_modules/,
                use: {
                    loader: "babel-loader",
                    options: {
                        presets: [
                            "@babel/preset-env",
                            "@babel/preset-react",
                            "@babel/preset-typescript",
                        ]
                    }
                }
            },
            {
                test: /\.css$/,
                exclude: /node_modules/,
                use: {
                    loader: "css-loader",
                },
            },
            {
                test: /\.svg$/,
                exclude: /node_modules/,
                use: {
                    loader: "svg-loader",
                }
            }
        ]
    },
    resolve: {
        extensions: [".tsx", ".ts", ".js"],
    },
    output: {
        path: path.resolve(__dirname, "./public"),
        filename: "bundle.js",
    },
};

export default config;
