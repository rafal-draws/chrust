FROM rafaldraws/python-ffmpeg:3.12
WORKDIR /app
COPY requirements.txt .
RUN pip install --no-cache-dir -r requirements.txt
COPY . .
CMD ["fastapi", "run", "app.py", "--port", "8888"]